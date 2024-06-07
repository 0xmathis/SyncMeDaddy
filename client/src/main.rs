use std::io::Result;
use std::net::{
    Ipv4Addr, TcpStream
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
use tcp::*;
use utils::to_valid_syncing_directory;
use std::path::{Path, PathBuf};

use smd_protocol::smd_packet::SMDpacket;

mod tcp;
mod utils;


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

fn main() -> Result<()> {
    init_logger();
    init_hooks();

    let args = Args::parse();
    let sync_directory: PathBuf = to_valid_syncing_directory(args.sync_directory)?;
    log::info!("Syncing directory {:?}", sync_directory);

    const IP: Ipv4Addr = Ipv4Addr::LOCALHOST;
    const PORT: u16 = 1234;
    const USERNAME: &str = "mathis";

    let stream: TcpStream = tcp::start_tcp_client(IP, PORT);
    connect(&stream, USERNAME)?;

    let packet: SMDpacket = SMDpacket::receive_from(&stream)?;

    if let SMDtype::Disconnect = packet.get_type() {
        disconnect(stream)?;
    }

    Ok(())
}
