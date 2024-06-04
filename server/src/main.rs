use std::io::Read;
use std::net::{
    Ipv4Addr, TcpListener, TcpStream
};
use std::panic;

mod tcp;


fn handle_connection(mut stream: TcpStream) {
    println!("Connection established : {:?}", stream);

    let mut n: usize = 1;
    let mut output: [u8; 128];

    while n > 0 {
        output = [0; 128];
        n = stream.read(&mut output).expect("Error reading");
        println!("{n} chars read: {}", String::from_utf8(output.to_vec()).expect(""));
    }
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
