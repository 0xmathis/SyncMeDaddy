use anyhow::{bail, Result};
use log::{info, warn};
use utils::read_file;
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
    let stream: TcpStream = TcpStream::connect(SocketAddr::from((ip, port)))?;

    Ok(stream)
}

pub fn connect(stream: &TcpStream, username: &str) -> Result<()> {
    let packet: SMDpacket = SMDpacket::new(1, SMDtype::Connect, Vec::from(username));
    packet.send_to(&stream)?;
    let response: SMDpacket = SMDpacket::receive_from(&stream)?;

    match response.get_type() {
        SMDtype::Connect => {
            let data = response.get_data();

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

    match response.get_type() {
        SMDtype::Update => {
            let data: &Vec<u8> = response.get_data();
            let update_answer: UpdateAnswer = UpdateAnswer::from_vec(data);

            return Ok(update_answer)
        },
        other => {
            bail!("Update refused by server, response type received: \"{other:?}\"");
        },
    };
}

pub fn upload(stream: &TcpStream, storage_directory: &PathBuf, to_upload: Files) -> Result<()> {
    assert!(storage_directory.is_absolute());
    info!("Upload started");

    let files: HashMap<PathBuf, File> = to_upload.get_data();

    for (filename, file) in files.into_iter() {
        let filepath: PathBuf = storage_directory.join(&filename);
        let buffer: Vec<u8> = read_file(filepath, file.get_size() as usize)?;

        let data_transfer: DataTransfer = DataTransfer::new(filename, file, buffer);
        SMDpacket::new(1, SMDtype::Upload, data_transfer.to_vec()).send_to(stream)?;
    }

    SMDpacket::new(1, SMDtype::Updated, Vec::new()).send_to(stream)?;
    info!("Upload finished");

    Ok(())
}

pub fn download(stream: &TcpStream, storage_directory: &PathBuf) -> Result<()> {
    assert!(storage_directory.is_absolute());
    info!("Download started");

    loop {
        let packet: SMDpacket = SMDpacket::receive_from(&stream)?;

        match packet.get_type() {
            SMDtype::Download => {
                let file: DataTransfer = DataTransfer::from_vec(packet.get_data());
                file.store(storage_directory)?;
            },
            SMDtype::Updated => break,
            other => {
                warn!("Invalid type received: \"{other:?}\"");
            },
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
