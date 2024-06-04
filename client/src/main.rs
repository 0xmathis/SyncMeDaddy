use std::net::{
    Ipv4Addr,
    TcpStream,
};
use std::panic;

use smd_protocol::smd_packet::SMDpacket;

mod tcp;

fn main() {
    panic::set_hook(Box::new(|e| {
        println!("When I panic I stop\n{e}");
    }));

    const IP: Ipv4Addr = Ipv4Addr::LOCALHOST;
    const PORT: u16 = 1234;

    let client: TcpStream = match tcp::start_tcp_client(IP, PORT) {
        Ok(server) => {
            println!("Client connected to {IP}:{PORT}");
            server
        }
        Err(e) => panic!("Error contacting server : {{ {e} }}"),
    };

    let packet: SMDpacket = SMDpacket::receive_from(client).expect("Error receiving");
    println!("Received packet : {packet}");
}
