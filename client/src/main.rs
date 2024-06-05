use std::net::{
    Ipv4Addr,
    TcpStream,
};
use std::panic;
use env_logger:: {
    Builder,
    Target,
};
use log;
use clap::Parser;
use path_absolutize::Absolutize;
use smd_protocol::smd_type::SMDtype;
use std::path::{Path, PathBuf};

use smd_protocol::smd_packet::SMDpacket;

mod tcp;


/// SMD Client
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Directory to synchronize
    sync_directory: String,
}


fn init_logger() {
    Builder::new()
        .target(Target::Stdout)
        .filter_level(log::LevelFilter::max())
        .format_level(true)
        .format_module_path(false)
        .format_indent(Some(4))
        .init();
}

fn init_hooks() {
    panic::set_hook(Box::new(|e| {
        println!("When I panic I stop\n{e}");
    }));
}

fn main() {
    init_logger();
    init_hooks();

    let args = Args::parse();
    let sync_directory: PathBuf = Path::new(&args.sync_directory).absolutize().unwrap().to_path_buf();
    log::info!("Syncing directory {:?}", sync_directory);

    const IP: Ipv4Addr = Ipv4Addr::LOCALHOST;
    const PORT: u16 = 1234;

    let stream: TcpStream = tcp::start_tcp_client(IP, PORT);

    let packet: SMDpacket = SMDpacket::new(1, SMDtype::DISCONNECT, Vec::from("Hello"));
    log::info!("To : {} | Sending : {}", stream.peer_addr().unwrap(), packet);
    let _ = packet.send_to(&stream);

    let packet: SMDpacket = SMDpacket::receive_from(&stream).expect("Error receiving");
    log::info!("From {} | Received {}", stream.peer_addr().unwrap(), packet);
}
