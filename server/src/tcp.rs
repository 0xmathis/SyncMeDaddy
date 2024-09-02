use anyhow::{bail, Result};
use log::{debug, info, warn};
use utils::read_file;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::Read;
use std::net::{Ipv4Addr, Shutdown, SocketAddr, TcpListener, TcpStream};
use std::path::PathBuf;

use smd_protocol::smd_packet::SMDpacket;
use smd_protocol::smd_type::SMDtype;
use utils::data_transfer::DataTransfer;
use utils::file::File;
use utils::files::Files;
use utils::update_answer::UpdateAnswer;

use crate::user::User;


pub fn start_tcp_server(ip: Ipv4Addr, port: u16) -> Result<TcpListener> {
    let server: TcpListener = TcpListener::bind(SocketAddr::from((ip, port)))?;

    Ok(server)
}

pub fn accept_smd_connect(packet: &SMDpacket) -> bool {
    if SMDtype::Connect.eq(packet.get_type()) {
        return packet.get_data().is_ascii();
    }

    false
}

pub fn handle_connection(stream: TcpStream, root_directory: &PathBuf) -> Result<()> {
    assert!(root_directory.is_absolute());

    info!("Connected to {:?}", stream.peer_addr());

    let user: User = match connect(&stream, root_directory) {
        Ok(user) => {
            info!("Connection successful to SMD client: {}", user.get_username());
            user
        }
        Err(e) => {
            bail!("Error connecting to client: {e}");
        },
    };

    let to_download: Files = match update(&stream, &user) {
        Ok(data) => {
            info!("{}: Update requested", user.get_username());
            data
        },
        Err(e) => {
            let _ = disconnect(&stream);
            bail!("Error updating client: {e}");
        },
    };

    upload(&stream, &user)?;
    download(&stream, &user, to_download)?;
    disconnect(&stream)
}

fn upload(stream: &TcpStream, user: &User) -> Result<()> {
    // TODO: Check that client uploaded all files ?

    let storage_directory: PathBuf = user.get_storage_directory();

    loop {
        let packet: SMDpacket = SMDpacket::receive_from(&stream)?;

        match *packet.get_type() {
            SMDtype::Upload => {
                let file: DataTransfer = DataTransfer::from_vec(packet.get_data());
                file.store(&storage_directory)?;
            },
            SMDtype::Updated => break,
            _ => bail!("Invalid type received while upload"),
        }
    };

    info!("{}: Upload finished", user.get_username());
    Ok(())
}

fn download(stream: &TcpStream, user: &User, to_download: Files) -> Result<()> {
    let files: HashMap<PathBuf, File> = to_download.get_data();
    let storage_directory: PathBuf = user.get_storage_directory();

    for (filename, file) in files.into_iter() {
        let filepath: PathBuf = storage_directory.join(&filename);
        let buffer: Vec<u8> = read_file(filepath, file.get_size() as usize)?;

        let data_transfer: DataTransfer = DataTransfer::new(filename, file, buffer);
        SMDpacket::new(1, SMDtype::Download, data_transfer.to_vec()).send_to(stream)?;
    }

    SMDpacket::new(1, SMDtype::Updated, Vec::new()).send_to(stream)?;
    info!("{}: Download finished", user.get_username());

    Ok(())
}

fn connect(stream: &TcpStream, root_directory: &PathBuf) -> Result<User> {
    assert!(root_directory.is_absolute());

    let packet: SMDpacket = SMDpacket::receive_from(&stream)?;

    if accept_smd_connect(&packet) {
        SMDpacket::new(1, SMDtype::Connect, Vec::from("OK")).send_to(&stream)?;
        info!("From: {:?} | Successfully connected", stream.peer_addr());
        return Ok(User::from_smd_packet(packet, root_directory));
    }

    warn!("From: {:?} | Received invalid Connect packet", stream.peer_addr());
    SMDpacket::new(1, SMDtype::Connect, Vec::from("KO")).send_to(&stream)?;
    bail!("Connection refused");
}

fn update(stream: &TcpStream, user: &User) -> Result<Files> {
    let packet: SMDpacket = SMDpacket::receive_from(&stream)?;
    let client_state: Files = Files::from_vec(packet.get_data());
    let stored_state: Files = user.get_state();

    let (to_upload, to_download): (Files, Files) = state_diff(stored_state, client_state);
    let update_answer: UpdateAnswer = UpdateAnswer::from_json(to_upload.clone(), to_download.clone());

    SMDpacket::new(1, SMDtype::Update, update_answer.to_vec()).send_to(&stream)?;

    debug!("{}: to_upload: {to_upload:?}", user.get_username());
    debug!("{}: to_download: {to_download:?}", user.get_username());

    Ok(to_download)
}

fn state_diff(server_data: Files, client_data: Files) -> (Files, Files) {
    /*
     * Upload means from client to server
     * Download means from server to client
     */

    let mut to_upload: HashMap<PathBuf, File> = HashMap::new();
    let mut to_download: HashMap<PathBuf, File> = HashMap::new();
    let server_data: HashMap<PathBuf, File> = server_data.get_data();
    let client_data: HashMap<PathBuf, File> = client_data.get_data();

    let mut filenames: HashSet<PathBuf> = HashSet::new();
    filenames.extend(server_data.keys().cloned());
    filenames.extend(client_data.keys().cloned());

    for filename in filenames.into_iter() {
        let server_contains: bool = server_data.contains_key(&filename);
        let client_contains: bool = client_data.contains_key(&filename);

        if server_contains && client_contains { // If both have the file stored
            let server_file: &File = server_data.get(&filename).unwrap();
            let client_file: &File = client_data.get(&filename).unwrap();

            if server_file.get_hash() != client_file.get_hash() {
                // If files are different, we keep the one modified last
                if server_file.get_mtime() < client_file.get_mtime() { // Last version on client
                    to_upload.insert(filename, client_file.clone());
                } else { // Last version on server
                    to_download.insert(filename, server_file.clone());
                }
            }
        } else if server_contains && !client_contains { // If only the server have the file stored
            let file: &File = server_data.get(&filename).unwrap();
            to_download.insert(filename, file.clone());
        } else if client_contains && !server_contains { // If only the client have the file stored
            let file: &File = client_data.get(&filename).unwrap();
            to_upload.insert(filename, file.clone());
        }
    }

    (Files::from_map(to_upload), Files::from_map(to_download))
}

fn disconnect(stream: &TcpStream) -> Result<()> {
    SMDpacket::new(1, SMDtype::Disconnect, Vec::new()).send_to(&stream)?;
    info!("Disconnected from {:?}", stream.peer_addr());
    stream.shutdown(Shutdown::Both)?;

    Ok(())
}
