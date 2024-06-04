use std::net::{
    Ipv4Addr,
    SocketAddr,
    TcpListener,
};

use std::io;


pub fn start_tcp_server(ip: Ipv4Addr, port: u16) -> io::Result<TcpListener> {
    let server = TcpListener::bind(SocketAddr::from((ip, port)))?;

    Ok(server)
}
