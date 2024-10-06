use clap::Parser;
use env_logger::{Builder, Target};
use log::{error, info, warn};
use std::net::{Ipv4Addr, TcpListener};
use std::panic;
use std::path::PathBuf;
use tcp::handle_connection;
use utils::to_valid_syncing_directory;

mod tcp;
mod user;


/// SMD Server
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
    let root_directory: PathBuf = to_valid_syncing_directory(args.sync_directory);
    info!("Syncing directory {:?}", root_directory);

    const IP: Ipv4Addr = Ipv4Addr::LOCALHOST;
    const PORT: u16 = 1234;

    let server: TcpListener = match tcp::start_tcp_server(IP, PORT) {
        Ok(server) => {
            info!("Server listening on {IP}:{PORT}");
            server
        },
        Err(e) => {
            error!("Error starting server: {e}");
            return ();
        },
    };

    for stream in server.incoming() {
        match stream {
            Ok(stream) => {
                if let Err(e) = handle_connection(stream, &root_directory) {
                    warn!("Error while handling connection: {e}");
                }
            },
            Err(e) => warn!("Encountered IO error: {e}"),
        };

        break;
    }
}
