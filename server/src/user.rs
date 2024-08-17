use std::path::{Path, PathBuf};
use std::fs;

use smd_protocol::smd_packet::SMDpacket;
use utils::my_json::Files;


pub struct User {
    username: String,
    sync_directory: PathBuf,
}

impl User {
    fn check_username(username: &String) -> bool {
        let username_path: &Path = Path::new(&username);

        if let Some(_) = username_path.parent() {
            return true;
        }

        false
    }

    pub fn new(username: String, root_directory: &PathBuf) -> Option<Self> {
        if !Self::check_username(&username) {
            return None;
        }

        let sync_directory: PathBuf = Self::build_sync_directory(root_directory, &username);
        let user: Self = Self { username, sync_directory };
        user.init_sync_directory();

        Some(user)
    }

    pub fn from_smd_packet(packet: SMDpacket, root_directory: &PathBuf) -> Self {
        let data: Vec<u8> = packet.get_data().clone();
        return Self::new(String::from_utf8(data).unwrap(), root_directory).unwrap()
    }

    fn init_sync_directory(&self) -> () {
        let sync_directory: &PathBuf = self.get_sync_directory();

        if !sync_directory.exists() {
            let _ = fs::create_dir(sync_directory);
        }
    }

    pub fn get_sync_directory(&self) -> &PathBuf {
        &self.sync_directory
    }

    pub fn get_state(&self) -> Files {
        let state: PathBuf = self.get_sync_directory().join("smd_state.json");
        Files::load_from_file(&state).unwrap()
    }

    pub fn build_sync_directory(root_directory: &PathBuf, username: &String) -> PathBuf {
        root_directory.join(username)
    }
}
