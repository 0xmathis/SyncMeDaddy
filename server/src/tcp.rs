use std::net::{
    Ipv4Addr, Shutdown, SocketAddr, TcpListener, TcpStream
};
use std::io::Result;

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
    match packet.get_type() {
        SMDtype::Connect => true,
        _ => false,
    }
}

pub fn handle_connection(stream: TcpStream) -> Result<()> {
    log::info!("Connected to {}", stream.peer_addr().unwrap());

    let packet: SMDpacket = SMDpacket::receive_from(&stream)?;

    if accept_smd_connect(&packet) {
        SMDpacket::new(1, SMDtype::Connect, Vec::from("OK")).send_to(&stream)?;
    } else {
        log::warn!("From : {} | Received invalid CONNECT packet", stream.peer_addr()?);
        SMDpacket::new(1, SMDtype::Connect, Vec::from("KO")).send_to(&stream)?;
    }
    
    disconnect(stream)
}

pub fn disconnect(stream: TcpStream) -> Result<()> {
    let packet: SMDpacket = SMDpacket::new(1, SMDtype::Disconnect, Vec::new());
    packet.send_to(&stream)?;
    stream.shutdown(Shutdown::Both)?;
    log::info!("Disconnected from {}", stream.peer_addr().unwrap());

    Ok(())
}
