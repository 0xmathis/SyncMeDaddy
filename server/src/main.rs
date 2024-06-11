use std::net::{
    Ipv4Addr, TcpListener
};
use std::io::Result;
use std::panic;
use env_logger:: {
    Builder,
    Target,
};
use log;
use clap::Parser;
use tcp::handle_connection;
use utils::to_valid_syncing_directory;
use std::path::PathBuf;

mod tcp;
mod user;


/// SMD Server
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
    let root_directory: PathBuf = to_valid_syncing_directory(args.sync_directory)?;
    log::info!("Syncing directory {:?}", root_directory);

    const IP: Ipv4Addr = Ipv4Addr::LOCALHOST;
    const PORT: u16 = 1234;

    let server: TcpListener = tcp::start_tcp_server(IP, PORT);

    for stream in server.incoming() {
        match stream {
            Ok(s) => {
                handle_connection(s, &root_directory)?;
            }
            Err(e) => panic!("Encountered IO error: {e}"),
        }
    }

    Ok(())
}
