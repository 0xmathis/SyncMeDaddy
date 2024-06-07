use std::net::{
    Ipv4Addr, Shutdown, SocketAddr, TcpListener, TcpStream
};
use std::io::{Error, ErrorKind, Result};

use smd_protocol::{smd_packet::SMDpacket, smd_type::SMDtype};


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
        return true;
    }
    
    false
}

pub fn handle_connection(stream: TcpStream) -> Result<()> {
    log::info!("Connected to {}", stream.peer_addr().unwrap());

    connect(&stream)?;

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

pub fn connect(stream: &TcpStream) -> Result<()> {
    let packet: SMDpacket = SMDpacket::receive_from(&stream)?;

    if accept_smd_connect(&packet) {
        SMDpacket::new(1, SMDtype::Connect, Vec::from("OK")).send_to(&stream)?;
        log::info!("From : {} | Successfully connected", stream.peer_addr()?);
        return Ok(());
    }

    log::warn!("From : {} | Received invalid Connect packet", stream.peer_addr()?);
    SMDpacket::new(1, SMDtype::Connect, Vec::from("KO")).send_to(&stream)?;
    Err(Error::new(ErrorKind::ConnectionRefused, "Connection refused"))
}

pub fn disconnect(stream: TcpStream) -> Result<()> {
    SMDpacket::new(1, SMDtype::Disconnect, Vec::new()).send_to(&stream)?;
    log::info!("Disconnected from {}", stream.peer_addr()?);
    stream.shutdown(Shutdown::Both)?;

    Ok(())
}
