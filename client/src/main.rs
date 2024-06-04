use std::io::Write;
use std::net::{
    Ipv4Addr,
    TcpStream,
};
use std::panic;

mod tcp;

fn send_message(mut client: TcpStream, input: &[u8]) {
    let n: usize = client.write(&input).expect("");
    println!("{n} characters sended to server");
}

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

    send_message(client, &Vec::from("Hello you"));
}
