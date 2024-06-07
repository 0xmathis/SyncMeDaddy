use std::net::{
    Ipv4Addr, Shutdown, SocketAddr, TcpStream
};
use std::io::{Error, ErrorKind, Result};

use smd_protocol::{smd_packet::SMDpacket, smd_type::SMDtype};


pub fn start_tcp_client(ip: Ipv4Addr, port: u16) -> TcpStream {
    match TcpStream::connect(SocketAddr::from((ip, port))) {
        Ok(stream) => {
            log::info!("Connected to {}:{}", ip, port);
            return stream;
        }
        Err(e) => panic!("Error contacting server : {{ {e} }}"),
    };
}

pub fn connect(stream: &TcpStream, username: &str) -> Result<()> {
    let packet: SMDpacket = SMDpacket::new(1, SMDtype::Connect, Vec::from(username));
    packet.send_to(&stream)?;

    let response: SMDpacket = SMDpacket::receive_from(&stream)?;

    match response.get_type() {
        SMDtype::Connect => {
            let data: &Vec<u8> = response.get_data();

            if *data == Vec::from("OK") {
                return Ok(());
            } else if *data == Vec::from("KO") {
                return Err(Error::new(ErrorKind::ConnectionRefused, "Connection refused"));
            } else {
                return Err(Error::new(ErrorKind::InvalidData, "Invalid data"));
            }
        },
        _ => return Err(Error::new(ErrorKind::InvalidData, "Unknown packet received")),
    }
}

pub fn disconnect(stream: TcpStream) -> Result<()> {
    log::info!("Disconnected from {}", stream.peer_addr()?);
    stream.shutdown(Shutdown::Both)?;

    Ok(())
}
