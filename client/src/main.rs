use clap::Parser;
use env_logger::{Builder, Target};
use log::{error, info};
use smd_protocol::smd_packet::SMDpacket;
use smd_protocol::smd_type::SMDtype;
use std::net::{Ipv4Addr, TcpStream};
use std::panic;
use std::path::PathBuf;
use tcp::{connect, delete, disconnect, download, update_request, upload};
use utils::files::Files;
use utils::update_answer::UpdateAnswer;
use utils::{get_current_state, to_valid_syncing_directory};

mod tcp;


/// SMD Client
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Directory to synchronize
    sync_directory: PathBuf,
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

fn main() -> () {
    init_logger();
    init_hooks();

    let args = Args::parse();
    let sync_directory: PathBuf = to_valid_syncing_directory(args.sync_directory);
    let storage_path: PathBuf = sync_directory.join("storage");
    let state_path: PathBuf = sync_directory.join("smd_state.json");
    info!("Syncing directory {:?}", sync_directory);

    const IP: Ipv4Addr = Ipv4Addr::LOCALHOST;
    const PORT: u16 = 1234;
    const USERNAME: &str = "user";

    let stored_state: Files = match Files::load_from_file(&state_path) {
        Ok(state) => {
            info!("Stored state loaded");
            state
        },
        Err(e) => {
            error!("Fail loading stored state: {e}");
            return ();
        }
    };

    let current_state: Files = match get_current_state(&storage_path, stored_state) {
        Ok(current_state) => {
            info!("Current state loaded");
            current_state
        },
        Err(e) => {
            error!("Error loading current state: {e}");
            return ();
        },
    };

    let stream: TcpStream = match tcp::start_tcp_client(IP, PORT) {
        Ok(stream) => {
            info!("Connected to {}:{}", IP, PORT);
            stream
        },
        Err(e) => {
            error!("Error starting tcp client: {e}");
            return ();
        },
    };

    match connect(&stream, USERNAME) {
        Ok(()) => info!("Connection successful to SMD server"),
        Err(e) => {
            error!("{e}");
            return ();
        },
    };

    let remote_diffs: UpdateAnswer = match update_request(&stream, &current_state) {
        Ok(remote_diffs) => {
            info!("Update accepted");
            remote_diffs
        },
        Err(e) => {
            error!("{e}");
            let _ = disconnect(&stream);
            return ();
        },
    };

    let (server_todo, client_todo): (Files, Files) = remote_diffs.data();

    if let Err(e) = delete(&storage_path, client_todo.clone()) {
        error!("{e}");
        return ();
    }

    if let Err(e) = upload(&stream, &storage_path, server_todo) {
        error!("{e}");
        return ();
    }

    if let Err(e) = download(&stream, &storage_path, client_todo) {
        error!("{e}");
        return ();
    }

    if let Ok(packet) = SMDpacket::receive_from(&stream) {
        if let SMDtype::Disconnect = packet.data_type() {
            let _ = disconnect(&stream);
        }
    }

    let final_state: Files = match get_current_state(&storage_path, current_state) {
        Ok(final_state) => {
            final_state
        },
        Err(e) => {
            error!("Error loading end state: {e}");
            return ();
        },
    };

    match final_state.store_to_file(&state_path) {
        Ok(()) => info!("State stored"),
        Err(e) => error!("Error storing state: {e}"),
    };
}
