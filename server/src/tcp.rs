use anyhow::{bail, Result};
use log::{debug, info, warn};
use utils::{get_current_state, read_file};
use utils::state::State;
use std::collections::{HashMap, HashSet};
use std::net::{Ipv4Addr, Shutdown, SocketAddr, TcpListener, TcpStream};
use std::path::PathBuf;

use smd_protocol::smd_packet::SMDpacket;
use smd_protocol::smd_type::SMDtype;
use utils::data_transfer::DataTransfer;
use utils::file::File;
use utils::files::Files;
use utils::update_answer::UpdateAnswer;

use crate::user::User;


pub fn start_tcp_server(ip: Ipv4Addr, port: u16) -> Result<TcpListener> {
    let server: TcpListener = TcpListener::bind(SocketAddr::from((ip, port)))?;

    Ok(server)
}

pub fn accept_smd_connect(packet: &SMDpacket) -> bool {
    if SMDtype::Connect.eq(packet.data_type()) {
        return packet.data().is_ascii();
    }

    false
}

pub fn handle_connection(stream: TcpStream, root_directory: &PathBuf) -> Result<()> {
    assert!(root_directory.is_absolute());

    info!("Connected to {:?}", stream.peer_addr());

    let user: User = match connect(&stream, root_directory) {
        Ok(user) => {
            info!("Connection successful to SMD client: {}", user.username());
            user
        }
        Err(e) => {
            bail!("Error connecting to client: {e}");
        },
    };

    let (server_todo, client_todo): (Files, Files) = match update(&stream, &user) {
        Ok(data) => {
            info!("{}: Update requested", user.username());
            data
        },
        Err(e) => {
            let _ = disconnect(&stream);
            bail!("Error updating client: {e}");
        },
    };

    delete(&user, &server_todo)?;
    upload(&stream, &user)?;
    download(&stream, &user, &client_todo)?;
    disconnect(&stream)?;
    user.store_state()
}

fn delete(user: &User, server_todo: &Files) -> Result<()> {
    info!("{}: Delete started", user.username());

    let files: &HashMap<PathBuf, File> = server_todo.data();
    let storage_directory: PathBuf = user.storage_directory();

    for (filename, file) in files.into_iter() {
        let filepath: PathBuf = storage_directory.join(&filename);

        if State::Deleted.ne(file.state()) {
            continue;
        }

        debug!("{}: Delete file \"{:?}\"", user.username(), filename);
    }

    info!("{}: Delete finished", user.username());

    Ok(())
}

fn upload(stream: &TcpStream, user: &User) -> Result<()> {
    info!("{}: Upload started", user.username());
    // TODO: Check that client uploaded all files ?

    let storage_directory: PathBuf = user.storage_directory();

    loop {
        let packet: SMDpacket = SMDpacket::receive_from(&stream)?;

        match packet.data_type() {
            SMDtype::Upload => {
                let file: DataTransfer = DataTransfer::from_vec(packet.data());
                file.store(&storage_directory)?;
            },
            SMDtype::Updated => break,
            _ => bail!("Invalid type received while upload"),
        }
    };

    info!("{}: Upload finished", user.username());
    Ok(())
}

fn download(stream: &TcpStream, user: &User, client_todo: &Files) -> Result<()> {
    info!("{}: Download started", user.username());

    let files: &HashMap<PathBuf, File> = client_todo.data();
    let storage_directory: PathBuf = user.storage_directory();

    for (filename, file) in files.into_iter() {
        let filepath: PathBuf = storage_directory.join(&filename);
        let buffer: Vec<u8> = read_file(filepath, file.size() as usize)?;

        let data_transfer: DataTransfer = DataTransfer::new(filename.clone(), file.clone(), buffer);
        SMDpacket::new(1, SMDtype::Download, data_transfer.to_vec()).send_to(stream)?;
    }

    SMDpacket::new(1, SMDtype::Updated, Vec::new()).send_to(stream)?;
    info!("{}: Download finished", user.username());

    Ok(())
}

fn connect(stream: &TcpStream, root_directory: &PathBuf) -> Result<User> {
    assert!(root_directory.is_absolute());

    let packet: SMDpacket = SMDpacket::receive_from(&stream)?;

    if accept_smd_connect(&packet) {
        SMDpacket::new(1, SMDtype::Connect, Vec::from("OK")).send_to(&stream)?;
        info!("From: {:?} | Successfully connected", stream.peer_addr());
        return Ok(User::from_smd_packet(packet, root_directory));
    }

    warn!("From: {:?} | Received invalid Connect packet", stream.peer_addr());
    SMDpacket::new(1, SMDtype::Connect, Vec::from("KO")).send_to(&stream)?;
    bail!("Connection refused");
}

fn update(stream: &TcpStream, user: &User) -> Result<(Files, Files)> {
    let packet: SMDpacket = SMDpacket::receive_from(&stream)?;
    let client_state: Files = Files::from_vec(packet.data());
    let stored_state: Files = user.state();

    let (server_todo, client_todo): (Files, Files) = state_diff(stored_state, client_state);

    let update_answer: UpdateAnswer = UpdateAnswer::from_json(server_todo.clone(), client_todo.clone());
    SMDpacket::new(1, SMDtype::Update, update_answer.to_vec()).send_to(&stream)?;

    Ok((server_todo, client_todo))
}

fn state_diff(server_data: Files, client_data: Files) -> (Files, Files) {
    /*
     * Upload means from client to server
     * Download means from server to client
     * server_todo means files the server needs to update
     * client_todo means files the client needs to update
     */

    let mut server_todo: HashMap<PathBuf, File> = HashMap::new();
    let mut client_todo: HashMap<PathBuf, File> = HashMap::new();
    let server_data: &HashMap<PathBuf, File> = server_data.data();
    let client_data: &HashMap<PathBuf, File> = client_data.data();

    let mut filenames: HashSet<PathBuf> = HashSet::new();
    filenames.extend(server_data.keys().cloned());
    filenames.extend(client_data.keys().cloned());

    for filename in filenames.into_iter() {
        let server_contains: bool = server_data.contains_key(&filename);
        let client_contains: bool = client_data.contains_key(&filename);

        if server_contains && client_contains { // If both have the file stored
            let server_file: &File = server_data.get(&filename).unwrap();
            let client_file: &File = client_data.get(&filename).unwrap();

            match (server_file.state(), client_file.state()) {
                (State::Unchanged, State::Unchanged) => {},
                (State::Unchanged, _) => {
                    server_todo.insert(filename, client_file.clone());
				},
                (_, State::Unchanged) => {
                    client_todo.insert(filename, server_file.clone());
				},
                (State::Created, State::Created) |
                (State::Created, State::Edited) |
                (State::Edited, State::Created) |
                (State::Edited, State::Edited) => {
                    if server_file.hash() != client_file.hash() {
                        // If files are different, we keep the one modified last
                        if server_file.mtime() < client_file.mtime() { // Last version on client
                            server_todo.insert(filename, client_file.clone());
                        } else { // Last version on server
                            client_todo.insert(filename, server_file.clone());
                        }
                    }
                },
                (State::Deleted, State::Deleted) => todo!(),
                (State::Edited, State::Deleted) => todo!(),
                (State::Deleted, State::Edited) => todo!(),
                (State::Deleted, State::Created) => todo!(),
                (State::Created, State::Deleted) => todo!(),
            }
        } else if server_contains && !client_contains { // If only the server have the file stored
            let server_file: &File = server_data.get(&filename).unwrap();

            if State::Unchanged.ne(server_file.state()) {
                client_todo.insert(filename, server_file.clone());
            }
        } else if client_contains && !server_contains { // If only the client have the file stored
            let client_file: &File = client_data.get(&filename).unwrap();

            if State::Unchanged.ne(client_file.state()) {
                server_todo.insert(filename, client_file.clone());
            }
        }
    }

    (Files::from_map(server_todo), Files::from_map(client_todo))
}

fn disconnect(stream: &TcpStream) -> Result<()> {
    SMDpacket::new(1, SMDtype::Disconnect, Vec::new()).send_to(&stream)?;
    info!("Disconnected from {:?}", stream.peer_addr());
    stream.shutdown(Shutdown::Both)?;

    Ok(())
}

#[cfg(test)]
mod server {
    use super::*;

    #[test]
    fn test_state_diff_unchanged_unchanged() {
        let mut server_input: HashMap<PathBuf, File> = HashMap::new();
        let mut client_input: HashMap<PathBuf, File> = HashMap::new();
        let server_output: HashMap<PathBuf, File> = HashMap::new();
        let client_output: HashMap<PathBuf, File> = HashMap::new();

        server_input.insert(PathBuf::from("file"), File::from_data(1, 2, [0; 20], State::Unchanged));
        client_input.insert(PathBuf::from("file"), File::from_data(2, 2, [0; 20], State::Unchanged));

        let server_data: Files = Files::from_map(server_input);
        let client_data: Files = Files::from_map(client_input);
        let server_todo: Files = Files::from_map(server_output);
        let client_todo: Files = Files::from_map(client_output);
        let result: (Files, Files) = state_diff(server_data, client_data);

        assert_eq!(result, (server_todo, client_todo));
    }

    #[test]
    fn test_state_diff_unchanged_created() {
        let mut server_input: HashMap<PathBuf, File> = HashMap::new();
        let mut client_input: HashMap<PathBuf, File> = HashMap::new();
        let mut server_output: HashMap<PathBuf, File> = HashMap::new();
        let client_output: HashMap<PathBuf, File> = HashMap::new();

        server_input.insert(PathBuf::from("file"), File::from_data(1, 2, [0; 20], State::Unchanged));
        client_input.insert(PathBuf::from("file"), File::from_data(2, 3, [1; 20], State::Created));
        server_output.insert(PathBuf::from("file"), File::from_data(2, 3, [1; 20], State::Created));

        let server_data: Files = Files::from_map(server_input);
        let client_data: Files = Files::from_map(client_input);
        let server_todo: Files = Files::from_map(server_output);
        let client_todo: Files = Files::from_map(client_output);
        let result: (Files, Files) = state_diff(server_data, client_data);

        assert_eq!(result, (server_todo, client_todo));
    }

    #[test]
    fn test_state_diff_unchanged_edited() {
        let mut server_input: HashMap<PathBuf, File> = HashMap::new();
        let mut client_input: HashMap<PathBuf, File> = HashMap::new();
        let mut server_output: HashMap<PathBuf, File> = HashMap::new();
        let client_output: HashMap<PathBuf, File> = HashMap::new();

        server_input.insert(PathBuf::from("file"), File::from_data(1, 2, [0; 20], State::Unchanged));
        client_input.insert(PathBuf::from("file"), File::from_data(2, 3, [1; 20], State::Edited));
        server_output.insert(PathBuf::from("file"), File::from_data(2, 3, [1; 20], State::Edited));

        let server_data: Files = Files::from_map(server_input);
        let client_data: Files = Files::from_map(client_input);
        let server_todo: Files = Files::from_map(server_output);
        let client_todo: Files = Files::from_map(client_output);
        let result: (Files, Files) = state_diff(server_data, client_data);

        assert_eq!(result, (server_todo, client_todo));
    }

    #[test]
    fn test_state_diff_unchanged_deleted() {
        let mut server_input: HashMap<PathBuf, File> = HashMap::new();
        let mut client_input: HashMap<PathBuf, File> = HashMap::new();
        let mut server_output: HashMap<PathBuf, File> = HashMap::new();
        let client_output: HashMap<PathBuf, File> = HashMap::new();

        server_input.insert(PathBuf::from("file"), File::from_data(1, 2, [0; 20], State::Unchanged));
        client_input.insert(PathBuf::from("file"), File::from_data(2, 3, [1; 20], State::Deleted));
        server_output.insert(PathBuf::from("file"), File::from_data(2, 3, [1; 20], State::Deleted));

        let server_data: Files = Files::from_map(server_input);
        let client_data: Files = Files::from_map(client_input);
        let server_todo: Files = Files::from_map(server_output);
        let client_todo: Files = Files::from_map(client_output);
        let result: (Files, Files) = state_diff(server_data, client_data);

        assert_eq!(result, (server_todo, client_todo));
    }

    #[test]
    fn test_state_diff_created_unchanged() {
        let mut server_input: HashMap<PathBuf, File> = HashMap::new();
        let mut client_input: HashMap<PathBuf, File> = HashMap::new();
        let server_output: HashMap<PathBuf, File> = HashMap::new();
        let mut client_output: HashMap<PathBuf, File> = HashMap::new();

        server_input.insert(PathBuf::from("file"), File::from_data(1, 2, [1; 20], State::Created));
        client_input.insert(PathBuf::from("file"), File::from_data(2, 3, [0; 20], State::Unchanged));
        client_output.insert(PathBuf::from("file"), File::from_data(1, 2, [1; 20], State::Created));

        let server_data: Files = Files::from_map(server_input);
        let client_data: Files = Files::from_map(client_input);
        let server_todo: Files = Files::from_map(server_output);
        let client_todo: Files = Files::from_map(client_output);
        let result: (Files, Files) = state_diff(server_data, client_data);

        assert_eq!(result, (server_todo, client_todo));
    }

    #[test]
    fn test_state_diff_created_created() {
        let mut server_input: HashMap<PathBuf, File> = HashMap::new();
        let mut client_input: HashMap<PathBuf, File> = HashMap::new();
        let server_output: HashMap<PathBuf, File> = HashMap::new();
        let client_output: HashMap<PathBuf, File> = HashMap::new();

        server_input.insert(PathBuf::from("file"), File::from_data(1, 2, [0; 20], State::Created));
        client_input.insert(PathBuf::from("file"), File::from_data(2, 3, [0; 20], State::Created));

        let server_data: Files = Files::from_map(server_input);
        let client_data: Files = Files::from_map(client_input);
        let server_todo: Files = Files::from_map(server_output);
        let client_todo: Files = Files::from_map(client_output);
        let result: (Files, Files) = state_diff(server_data, client_data);

        assert_eq!(result, (server_todo, client_todo));
    }

    #[test]
    fn test_state_diff_created_edited() {
        let mut server_input: HashMap<PathBuf, File> = HashMap::new();
        let mut client_input: HashMap<PathBuf, File> = HashMap::new();
        let server_output: HashMap<PathBuf, File> = HashMap::new();
        let mut client_output: HashMap<PathBuf, File> = HashMap::new();

        server_input.insert(PathBuf::from("file"), File::from_data(2, 3, [0; 20], State::Created));
        client_input.insert(PathBuf::from("file"), File::from_data(1, 2, [1; 20], State::Edited));
        client_output.insert(PathBuf::from("file"), File::from_data(2, 3, [0; 20], State::Created));

        let server_data: Files = Files::from_map(server_input);
        let client_data: Files = Files::from_map(client_input);
        let server_todo: Files = Files::from_map(server_output);
        let client_todo: Files = Files::from_map(client_output);
        let result: (Files, Files) = state_diff(server_data, client_data);

        assert_eq!(result, (server_todo, client_todo));
    }

    #[ignore = "not implemented"]
    #[test]
    fn test_state_diff_created_deleted() {
        todo!();
    }

    #[test]
    fn test_state_diff_edited_unchanged() {
        let mut server_input: HashMap<PathBuf, File> = HashMap::new();
        let mut client_input: HashMap<PathBuf, File> = HashMap::new();
        let server_output: HashMap<PathBuf, File> = HashMap::new();
        let mut client_output: HashMap<PathBuf, File> = HashMap::new();

        server_input.insert(PathBuf::from("file"), File::from_data(1, 2, [1; 20], State::Edited));
        client_input.insert(PathBuf::from("file"), File::from_data(2, 2, [0; 20], State::Unchanged));
        client_output.insert(PathBuf::from("file"), File::from_data(1, 2, [1; 20], State::Edited));

        let server_data: Files = Files::from_map(server_input);
        let client_data: Files = Files::from_map(client_input);
        let server_todo: Files = Files::from_map(server_output);
        let client_todo: Files = Files::from_map(client_output);
        let result: (Files, Files) = state_diff(server_data, client_data);

        assert_eq!(result, (server_todo, client_todo));
    }

    #[test]
    fn test_state_diff_edited_created() {
        let mut server_input: HashMap<PathBuf, File> = HashMap::new();
        let mut client_input: HashMap<PathBuf, File> = HashMap::new();
        let mut server_output: HashMap<PathBuf, File> = HashMap::new();
        let client_output: HashMap<PathBuf, File> = HashMap::new();

        server_input.insert(PathBuf::from("file"), File::from_data(1, 2, [0; 20], State::Edited));
        client_input.insert(PathBuf::from("file"), File::from_data(2, 3, [1; 20], State::Created));
        server_output.insert(PathBuf::from("file"), File::from_data(2, 3, [1; 20], State::Created));

        let server_data: Files = Files::from_map(server_input);
        let client_data: Files = Files::from_map(client_input);
        let server_todo: Files = Files::from_map(server_output);
        let client_todo: Files = Files::from_map(client_output);
        let result: (Files, Files) = state_diff(server_data, client_data);

        assert_eq!(result, (server_todo, client_todo));
    }

    #[test]
    fn test_state_diff_edited_edited() {
        let mut server_input: HashMap<PathBuf, File> = HashMap::new();
        let mut client_input: HashMap<PathBuf, File> = HashMap::new();
        let mut server_output: HashMap<PathBuf, File> = HashMap::new();
        let client_output: HashMap<PathBuf, File> = HashMap::new();

        server_input.insert(PathBuf::from("file"), File::from_data(1, 2, [0; 20], State::Edited));
        client_input.insert(PathBuf::from("file"), File::from_data(2, 3, [1; 20], State::Edited));
        server_output.insert(PathBuf::from("file"), File::from_data(2, 3, [1; 20], State::Edited));

        let server_data: Files = Files::from_map(server_input);
        let client_data: Files = Files::from_map(client_input);
        let server_todo: Files = Files::from_map(server_output);
        let client_todo: Files = Files::from_map(client_output);
        let result: (Files, Files) = state_diff(server_data, client_data);

        assert_eq!(result, (server_todo, client_todo));
    }
    
    #[ignore = "not implemented"]
    #[test]
    fn test_state_diff_edited_deleted() {
        todo!();
    }

    #[ignore = "not implemented"]
    #[test]
    fn test_state_diff_deleted_unchanged() {
        todo!();
    }   

    #[ignore = "not implemented"]
    #[test]
    fn test_state_diff_deleted_created() {
        todo!();
    }

    #[ignore = "not implemented"]
    #[test]
    fn test_state_diff_deleted_edited() {
        todo!();
    }

    #[ignore = "not implemented"]
    #[test]
    fn test_state_diff_deleted_deleted() {
        todo!();
    }
}
