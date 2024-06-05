use std::net::{
    Ipv4Addr,
    SocketAddr,
    TcpStream,
};


pub fn start_tcp_client(ip: Ipv4Addr, port: u16) -> TcpStream {
    match TcpStream::connect(SocketAddr::from((ip, port))) {
        Ok(stream) => {
            log::info!("Connected to {ip}:{port}");
            return stream;
        }
        Err(e) => panic!("Error contacting server : {{ {e} }}"),
    };
}

