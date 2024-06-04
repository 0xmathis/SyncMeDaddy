use std::net::{
    Ipv4Addr, TcpListener, TcpStream
};
use std::panic;

use smd_protocol::smd_packet::SMDpacket;
use smd_protocol::smd_type::SMDtype;

mod tcp;


fn handle_connection(stream: TcpStream) {
    println!("Connection established : {:?}", stream);

    let packet: SMDpacket = SMDpacket::new(1, SMDtype::CONNECT, Vec::from("Hello"));
    println!("Sending packet : {}", packet);
    let _ = packet.send_to(stream);
}

fn main() {
    panic::set_hook(Box::new(|e| {
        println!("When I panic I stop\n{e}");
    }));

    const IP: Ipv4Addr = Ipv4Addr::LOCALHOST;
    const PORT: u16 = 1234;

    let server: TcpListener = match tcp::start_tcp_server(IP, PORT) {
        Ok(server) => {
            println!("Server listening on {IP}:{PORT}");
            server
        }
        Err(e) => panic!("Error starting server : {{ {e} }}"),
    };

    for stream in server.incoming() {
        match stream {
            Ok(s) => {
                handle_connection(s);
            }
            Err(e) => panic!("Encountered IO error: {e}"),
        }
    }
}
