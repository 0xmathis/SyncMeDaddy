use std::collections::{HashMap, HashSet};
use std::net::{
    Ipv4Addr, Shutdown, SocketAddr, TcpListener, TcpStream
};
use std::io::{Error, ErrorKind, Result};
use std::path::PathBuf;

use smd_protocol::{smd_packet::SMDpacket, smd_type::SMDtype};
use utils::get_current_state;
use utils::my_json::{File, JSON};

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
    update(&stream, &user)?;

    // loop {
    //     let packet: SMDpacket = SMDpacket::receive_from(&stream)?;

    //     match packet.get_type() {
    //         SMDtype::Disconnect => break,
    //         SMDtype::UpdateRequest => {}
    //         SMDtype::Update => {}
    //         SMDtype::Updated => {}
    //         SMDtype::Upload => {}
    //         SMDtype::Download => {}
    //         _ => {}
    //     }
    // }

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

pub fn update(stream: &TcpStream, user: &User) -> Result<(JSON, JSON)> {
    let packet: SMDpacket = SMDpacket::receive_from(&stream)?;
    let client_data: JSON = JSON::from_vec(packet.get_data());
    let sync_directory: &PathBuf = user.get_sync_directory();
    println!("{:?}", client_data);

    let data: JSON = get_current_state(sync_directory)?;
    println!("{:?}", data);

    let (to_upload, to_download): (JSON, JSON) = state_diff(data, client_data);

    Ok((to_upload, to_download))
}

fn state_diff(server_data: JSON, client_data: JSON) -> (JSON, JSON) {
    let mut to_upload: HashMap<String, File> = HashMap::new();
    let mut to_download: HashMap<String, File> = HashMap::new();
    let server_data: &HashMap<String, File> = server_data.get_data();
    let client_data: &HashMap<String, File> = client_data.get_data();

    let mut files: HashSet<&String> = HashSet::new();
    files.extend(server_data.keys());
    files.extend(client_data.keys());

    for file in files {
        println!("{}", file);
        let server_contains: bool = server_data.contains_key(file);
        let client_contains: bool = client_data.contains_key(file);

        if server_contains && client_contains {
            // TODO
        } else if server_contains {
            to_upload.insert(file.to_string(), server_data.get(file).unwrap().to_owned());
        } else if client_contains {
            to_download.insert(file.to_string(), client_data.get(file).unwrap().to_owned());
        }
        
    }

    (JSON::from_map(to_upload), JSON::from_map(to_download))
}

pub fn disconnect(stream: TcpStream) -> Result<()> {
    SMDpacket::new(1, SMDtype::Disconnect, Vec::new()).send_to(&stream)?;
    log::info!("Disconnected from {}", stream.peer_addr()?);
    stream.shutdown(Shutdown::Both)?;

    Ok(())
}
