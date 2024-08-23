use std::{fs, u8};
use std::io::Read;
use std::collections::HashMap;
use std::net::{
    Ipv4Addr,
    Shutdown,
    SocketAddr,
    TcpStream,
};
use std::io::{
    Error,
    ErrorKind,
    Result
};
use std::path::PathBuf;

use smd_protocol::smd_packet::SMDpacket;
use smd_protocol::smd_type::SMDtype;
use utils::my_json::{DataTransfer, File, Files, UpdateAnswer};


pub fn start_tcp_client(ip: Ipv4Addr, port: u16) -> TcpStream {
    match TcpStream::connect(SocketAddr::from((ip, port))) {
        Ok(stream) => {
            log::info!("Connected to {}:{}", ip, port);
            return stream;
        }
        Err(e) => panic!("Error contacting server : {{ {e} }}"),
    };
}

pub fn connect(stream: &TcpStream, username: &str) -> Result<()> {
    let packet: SMDpacket = SMDpacket::new(1, SMDtype::Connect, Vec::from(username));
    packet.send_to(&stream)?;

    let response: SMDpacket = SMDpacket::receive_from(&stream)?;

    match response.get_type() {
        SMDtype::Connect => {
            let data: &Vec<u8> = response.get_data();

            if *data == Vec::from("OK") {
                return Ok(());
            } else if *data == Vec::from("KO") {
                return Err(Error::new(ErrorKind::ConnectionRefused, "Connection refused"));
            } else {
                return Err(Error::new(ErrorKind::InvalidData, "Invalid data"));
            }
        },
        _ => return Err(Error::new(ErrorKind::InvalidData, "Unknown packet received")),
    };
}

pub fn update_request(stream: &TcpStream, current_state: Files) -> Result<UpdateAnswer> {
    let data: Vec<u8> = current_state.to_vec();

    let packet: SMDpacket = SMDpacket::new(1, SMDtype::UpdateRequest, data);
    packet.send_to(&stream)?;

    let response: SMDpacket = SMDpacket::receive_from(&stream)?;

    match response.get_type() {
        SMDtype::Update => {
            let data: &Vec<u8> = response.get_data();
            let json: UpdateAnswer = UpdateAnswer::from_vec(data);

            return Ok(json)
        },
        _ => return Err(Error::new(ErrorKind::InvalidData, "Unknown packet received")),
    };
}

pub fn upload(stream: &TcpStream, storage_directory: &PathBuf, to_upload: Files) -> Result<()> {
    assert!(storage_directory.is_absolute());

    let files: HashMap<PathBuf, File> = to_upload.get_data();

    for (filename, file) in files.iter() {
        let filepath: PathBuf = storage_directory.join(filename);

        let mut buffer: Vec<u8> = Vec::new();
        buffer.resize(file.get_size() as usize, 0);
        let mut file_reader: fs::File = fs::File::open(filepath)?;
        file_reader.read_exact(&mut buffer)?;

        let data_transfer: DataTransfer = DataTransfer::new(filename.clone(), file.clone(), buffer);
        SMDpacket::new(1, SMDtype::Upload, data_transfer.to_vec()).send_to(stream)?;
    }

    SMDpacket::new(1, SMDtype::Updated, Vec::new()).send_to(stream)?;
    log::info!("Upload finished");

    Ok(())
}

pub fn download(stream: &TcpStream, storage_directory: &PathBuf, to_download: Files) -> Result<()> {
    assert!(storage_directory.is_absolute());

    loop {
        let packet: SMDpacket = SMDpacket::receive_from(&stream)?;

        match *packet.get_type() {
            SMDtype::Download => {
                let file: DataTransfer = DataTransfer::from_vec(packet.get_data());
                file.store(storage_directory)?;
            },
            SMDtype::Updated => break,
            _ => return Err(Error::new(ErrorKind::InvalidData, "Invalid type received while upload")),
        }
    };

    log::info!("Download finished");
    Ok(())
}

pub fn disconnect(stream: TcpStream) -> Result<()> {
    log::info!("Disconnected from {}", stream.peer_addr()?);
    stream.shutdown(Shutdown::Both)?;

    Ok(())
}
