use anyhow::{bail, Context, Result};
use log::{debug, info, warn};
use utils::read_file;
use utils::state::State;
use std::collections::HashMap;
use std::net::{Ipv4Addr, Shutdown, SocketAddr, TcpStream};
use std::path::PathBuf;

use smd_protocol::smd_packet::SMDpacket;
use smd_protocol::smd_type::SMDtype;
use utils::data_transfer::DataTransfer;
use utils::file::File;
use utils::files::Files;
use utils::update_answer::UpdateAnswer;


pub fn start_tcp_client(ip: Ipv4Addr, port: u16) -> Result<TcpStream> {
    TcpStream::connect(SocketAddr::from((ip, port))).context("Unable to start TCP client")
}

pub fn connect(stream: &TcpStream, username: &str) -> Result<()> {
    let packet: SMDpacket = SMDpacket::new(1, SMDtype::Connect, Vec::from(username));
    packet.send_to(&stream)?;
    let response: SMDpacket = SMDpacket::receive_from(&stream)?;

    match response.data_type() {
        SMDtype::Connect => {
            let data = response.data();

            if Vec::from("OK").eq(data) {
                return Ok(());
            }

            bail!("Connection refused by server, data received: \"{data:?}\"");
        },
        other => {
            bail!("Connection refused by server, response type received: \"{other:?}\"");
        },
    };
}

pub fn update_request(stream: &TcpStream, current_state: &Files) -> Result<UpdateAnswer> {
    let data: Vec<u8> = current_state.to_vec();
    let packet: SMDpacket = SMDpacket::new(1, SMDtype::UpdateRequest, data);
    packet.send_to(&stream)?;
    let response: SMDpacket = SMDpacket::receive_from(&stream)?;

    match response.data_type() {
        SMDtype::Update => {
            let data: Vec<u8> = response.data().to_vec();
            let update_answer: UpdateAnswer = UpdateAnswer::from_vec(data)?;

            return Ok(update_answer)
        },
        other => {
            bail!("Update refused by server, response type received: \"{other:?}\"");
        },
    };
}

pub fn delete(storage_directory: &PathBuf, client_todo: &Files) -> Result<()> {
    assert!(storage_directory.is_absolute());
    info!("Delete started");

    let files: &HashMap<PathBuf, File> = client_todo.data();

    for (filename, file) in files.into_iter() {
        let filepath: PathBuf = storage_directory.join(&filename);

        if State::Deleted.ne(file.state()) {
            continue;
        }

        debug!("Delete file \"{:?}\"", filename);
    }

    info!("Delete finished");

    Ok(())
}

pub fn upload(stream: &TcpStream, storage_directory: &PathBuf, server_todo: &Files) -> Result<()> {
    // Upload means from client to server
    assert!(storage_directory.is_absolute());
    info!("Upload started");

    let files: &HashMap<PathBuf, File> = server_todo.data();

    for (filename, file) in files.into_iter() {
        let filepath: PathBuf = storage_directory.join(&filename);
        let buffer: Vec<u8> = read_file(filepath, file.size() as usize)?;

        let data_transfer: DataTransfer = DataTransfer::new(filename.clone(), file.clone(), buffer);
        SMDpacket::new(1, SMDtype::Upload, data_transfer.to_vec()).send_to(stream)?;
    }

    SMDpacket::new(1, SMDtype::Updated, Vec::new()).send_to(stream)?;
    info!("Upload finished");

    Ok(())
}

pub fn download(stream: &TcpStream, storage_directory: &PathBuf, client_todo: &Files) -> Result<()> {
    // Download means from server to client
    assert!(storage_directory.is_absolute());
    info!("Download started");
    
    let client_todo: &HashMap<PathBuf, File> = client_todo.data();

    loop {
        let packet: SMDpacket = SMDpacket::receive_from(&stream)?;

        match packet.data_type() {
            SMDtype::Download => {
                let data: DataTransfer = match DataTransfer::from_vec(packet.data()) {
                    Ok(data) => data,
                    Err(e) => {
                        warn!("Error during downloading: {e}");
                        continue;
                    }
                };

                if client_todo.contains_key(data.filename()) {
                    data.store(&storage_directory)?;
                } else {
                    warn!("Unknown file received");
                }
            },
            SMDtype::Updated => break,
            other => warn!("Invalid type received: \"{other:?}\""),
        };
    };

    info!("Download finished");
    Ok(())
}

pub fn disconnect(stream: &TcpStream) -> Result<()> {
    stream.shutdown(Shutdown::Both)?;

    info!("Disconnected");
    Ok(())
}
