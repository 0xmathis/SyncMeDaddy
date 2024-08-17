use std::collections::{HashMap, HashSet};
use std::net::{
    Ipv4Addr, Shutdown, SocketAddr, TcpListener, TcpStream
};
use std::io::{Error, ErrorKind, Result};
use std::path::PathBuf;

use smd_protocol::{smd_packet::SMDpacket, smd_type::SMDtype};
use utils::my_json::{File, Files, UpdateAnswer};

use crate::user::User;


pub fn start_tcp_server(ip: Ipv4Addr, port: u16) -> TcpListener {
    match TcpListener::bind(SocketAddr::from((ip, port))) {
        Ok(server) => {
            log::info!("Server listening on {ip}:{port}");
            return server;
        }
        Err(e) => panic!("Error starting server : {{ {e} }}"),
    };
}

pub fn accept_smd_connect(packet: &SMDpacket) -> bool {
    if let SMDtype::Connect = packet.get_type() {
        if packet.get_data().is_ascii() {
            return true;
        }
    }

    false
}

pub fn handle_connection(stream: TcpStream, root_directory: &PathBuf) -> Result<()> {
    log::info!("Connected to {}", stream.peer_addr().unwrap());

    let user: User = connect(&stream, root_directory)?;
    update(&stream, user)?;

    // loop {
    //     let packet: SMDpacket = SMDpacket::receive_from(&stream)?;

    //     match packet.get_type() {
    //         SMDtype::Disconnect => break,
    //         SMDtype::Upload => {}
    //         SMDtype::Updated => break,
    //         _ => {}
    //     };
    // };

    disconnect(stream)
}

pub fn connect(stream: &TcpStream, root_directory: &PathBuf) -> Result<User> {
    let packet: SMDpacket = SMDpacket::receive_from(&stream)?;

    if accept_smd_connect(&packet) {
        SMDpacket::new(1, SMDtype::Connect, Vec::from("OK")).send_to(&stream)?;
        log::info!("From : {} | Successfully connected", stream.peer_addr()?);
        return Ok(User::from_smd_packet(packet, root_directory));
    }

    log::warn!("From : {} | Received invalid Connect packet", stream.peer_addr()?);
    SMDpacket::new(1, SMDtype::Connect, Vec::from("KO")).send_to(&stream)?;
    Err(Error::new(ErrorKind::ConnectionRefused, "Connection refused"))
}

pub fn update(stream: &TcpStream, user: User) -> Result<()> {
    let packet: SMDpacket = SMDpacket::receive_from(&stream)?;
    let client_state: Files = Files::from_vec(packet.get_data());
    let sync_directory: &PathBuf = user.get_sync_directory();

    let stored_state: Files = user.get_state();
    println!("Client : {client_state:?}");
    println!("Server : {stored_state:?}");

    let (to_upload, to_download): (Files, Files) = state_diff(stored_state, client_state);
    let update_answer: UpdateAnswer = UpdateAnswer::from_json(to_upload, to_download);

    SMDpacket::new(1, SMDtype::Update, update_answer.to_vec()).send_to(&stream)?;

    Ok(())
}

fn state_diff(server_data: Files, client_data: Files) -> (Files, Files) {
    let mut to_upload: HashMap<PathBuf, File> = HashMap::new();
    let mut to_download: HashMap<PathBuf, File> = HashMap::new();
    let server_data: HashMap<PathBuf, File> = server_data.get_data();
    let client_data: HashMap<PathBuf, File> = client_data.get_data();

    let mut filenames: HashSet<&PathBuf> = HashSet::new();
    filenames.extend(server_data.keys());
    filenames.extend(client_data.keys());

    for filename in filenames {
        println!("{:?}", filename);
        let server_contains: bool = server_data.contains_key(filename);
        let client_contains: bool = client_data.contains_key(filename);

        if server_contains && client_contains {
            // TODO
            let server_file: File = server_data.get(filename).unwrap().to_owned();
            let client_file: File = client_data.get(filename).unwrap().to_owned();

            if server_file.get_hash() != client_file.get_hash() {
                if server_file.get_mtime() < client_file.get_mtime() {
                } else {
                }
            }
        } else if server_contains && !client_contains {
            let file: File = server_data.get(filename).unwrap().to_owned();
            to_upload.insert(filename.clone(), file);
        } else if client_contains && !server_contains {
            let file: File = client_data.get(filename).unwrap().to_owned();
            to_download.insert(filename.clone(), file);
        }
    }

    (Files::from_map(to_upload), Files::from_map(to_download))
}

pub fn disconnect(stream: TcpStream) -> Result<()> {
    SMDpacket::new(1, SMDtype::Disconnect, Vec::new()).send_to(&stream)?;
    log::info!("Disconnected from {}", stream.peer_addr()?);
    stream.shutdown(Shutdown::Both)?;

    Ok(())
}
