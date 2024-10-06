use anyhow::{bail, Context, Result};
use log::{debug, info, warn};
use utils::read_file;
use utils::state::State;
use std::cell::RefCell;
use std::collections::HashMap;
use std::net::{Ipv4Addr, Shutdown, SocketAddr, TcpListener, TcpStream};
use std::path::PathBuf;
use std::rc::Rc;

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
    if SMDtype::Connect.eq(packet.data_type()) {
        return packet.data().is_ascii();
    }

    false
}

pub fn handle_connection(stream: TcpStream, root_directory: &PathBuf) -> Result<()> {
    assert!(root_directory.is_absolute());

    info!("Connected to {:?}", stream.peer_addr());

    let user: User = match connect(&stream, root_directory) {
        Ok(user) => {
            info!("Connection successful to SMD client: {}", user.username());
            user
        }
        Err(e) => {
            bail!("Error connecting to client: {e}");
        },
    };

    let (server_todo, client_todo): (Files, Files) = match update(&stream, &user) {
        Ok(data) => {
            info!("{}: Update requested", user.username());
            data
        },
        Err(e) => {
            let _ = disconnect(&stream);
            bail!("Error updating client: {e}");
        },
    };

    delete(&user, server_todo.clone())?;
    upload(&stream, &user, server_todo)?;
    download(&stream, &user, client_todo)?;
    disconnect(&stream)?;
    user.store_state()
}

fn connect(stream: &TcpStream, root_directory: &PathBuf) -> Result<User> {
    assert!(root_directory.is_absolute());

    let packet: SMDpacket = SMDpacket::receive_from(&stream)?;

    if accept_smd_connect(&packet) {
        SMDpacket::new(1, SMDtype::Connect, Vec::from("OK")).send_to(&stream)?;
        info!("From: {:?} | Successfully connected", stream.peer_addr());
        return User::from_smd_packet(packet, root_directory);
    }

    warn!("From: {:?} | Received invalid Connect packet", stream.peer_addr());
    SMDpacket::new(1, SMDtype::Connect, Vec::from("KO")).send_to(&stream)?;
    bail!("Connection refused");
}

fn update(stream: &TcpStream, user: &User) -> Result<(Files, Files)> {
    let packet: SMDpacket = SMDpacket::receive_from(&stream)?;
    let client_state: Files = Files::from_vec(packet.data())?;
    let stored_state: Files = user.state();

    let (server_todo, client_todo): (Files, Files) = Files::diff(stored_state, client_state);

    let update_answer: UpdateAnswer = UpdateAnswer::from_json(server_todo.clone(), client_todo.clone());
    SMDpacket::new(1, SMDtype::Update, update_answer.to_vec()).send_to(&stream)?;

    Ok((server_todo, client_todo))
}

fn delete(user: &User, server_todo: Files) -> Result<()> {
    info!("{}: Delete started", user.username());

    let files: &HashMap<Rc<PathBuf>, Rc<RefCell<File>>> = server_todo.data();
    let storage_directory: PathBuf = user.storage_directory();

    for (filename, file) in files.into_iter() {
        let filepath: PathBuf = storage_directory.join(filename.to_path_buf());

            if State::Deleted.ne(file.borrow().state()) {
            continue;
        }

        debug!("{}: Delete file \"{:?}\"", user.username(), filename);
    }

    info!("{}: Delete finished", user.username());

    Ok(())
}

fn upload(stream: &TcpStream, user: &User, server_todo: Files) -> Result<()> {
    // Upload means from client to server
    info!("{}: Upload started", user.username());
    // TODO: Check that client uploaded all files ?

    let storage_directory: PathBuf = user.storage_directory();
    let server_todo: &HashMap<Rc<PathBuf>, Rc<RefCell<File>>> = server_todo.data();

    loop {
        let packet: SMDpacket = SMDpacket::receive_from(&stream)?;

        match packet.data_type() {
            SMDtype::Upload => {
                let data: DataTransfer = match DataTransfer::from_vec(packet.data()) {
                    Ok(data) => data,
                    Err(e) => {
                        warn!("Error during uploading: {e}");
                        continue;
                    }
                };

                if server_todo.contains_key(data.filename()) {
                    data.store(&storage_directory)?;
                } else {
                    warn!("{}: Unknown file received", user.username());
                }
            },
            SMDtype::Updated => break,
            _ => bail!("Invalid type received while upload"),
        }
    };

    info!("{}: Upload finished", user.username());
    Ok(())
}

fn download(stream: &TcpStream, user: &User, client_todo: Files) -> Result<()> {
    // Download means from server to client
    info!("{}: Download started", user.username());

    let files: &HashMap<Rc<PathBuf>, Rc<RefCell<File>>> = client_todo.data();
    let storage_directory: PathBuf = user.storage_directory();

    for (filename, file) in files.into_iter() {
        let filepath: PathBuf = storage_directory.join(filename.to_path_buf());
        let buffer: Vec<u8> = read_file(filepath, file.borrow().size() as usize)?;

        let data_transfer: DataTransfer = DataTransfer::new(Rc::clone(filename), Rc::clone(file), buffer);
        SMDpacket::new(1, SMDtype::Download, data_transfer.to_vec()).send_to(stream)?;
    }

    SMDpacket::new(1, SMDtype::Updated, Vec::new()).send_to(stream)?;
    info!("{}: Download finished", user.username());

    Ok(())
}

fn disconnect(stream: &TcpStream) -> Result<()> {
    SMDpacket::new(1, SMDtype::Disconnect, Vec::new()).send_to(&stream)?;
    info!("Disconnected from {:?}", stream.peer_addr());
    stream.shutdown(Shutdown::Both).context("Unable to shutdown the connection")
}
