use std::io::Result;
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
use smd_protocol::smd_type::SMDtype;
use tcp::*;
use utils::my_json::{UpdateAnswer, Files};
use utils::{get_current_state, to_valid_syncing_directory};
use std::path::PathBuf;

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

fn main() -> Result<()> {
    init_logger();
    init_hooks();

    let args = Args::parse();
    let sync_directory: PathBuf = to_valid_syncing_directory(args.sync_directory)?;
    let storage: PathBuf = sync_directory.join("storage");
    let state: PathBuf = sync_directory.join("smd_state.json");
    log::info!("Syncing directory {:?}", sync_directory);

    const IP: Ipv4Addr = Ipv4Addr::LOCALHOST;
    const PORT: u16 = 1234;
    const USERNAME: &str = "user";

    let stored_state: Files = Files::load_from_file(&state).unwrap();
    let current_state: Files = get_current_state(&storage, stored_state).unwrap();

    let stream: TcpStream = tcp::start_tcp_client(IP, PORT);
    connect(&stream, USERNAME)?;
    let remote_diffs: UpdateAnswer = update_request(&stream, current_state)?;

    let packet: SMDpacket = SMDpacket::receive_from(&stream)?;

    if let SMDtype::Disconnect = packet.get_type() {
        disconnect(stream)?;
    }

    Ok(())
}
