use std::net::{
    Ipv4Addr,
    SocketAddr,
    TcpStream,
};

use std::io;


pub fn start_tcp_client(ip: Ipv4Addr, port: u16) -> io::Result<TcpStream> {
    let client = TcpStream::connect(SocketAddr::from((ip, port)))?;

    Ok(client)
}

