use std::path::{Path, PathBuf};
use std::fs;

use smd_protocol::smd_packet::SMDpacket;
use utils::get_current_state;
use utils::files::Files;


pub struct User {
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
        assert!(root_directory.is_absolute());

        if !Self::check_username(&username) {
            return None;
        }

        let sync_directory: PathBuf = root_directory.join(&username);
        let user: Self = Self { sync_directory };
        user.init_sync_directory();

        Some(user)
    }

    pub fn from_smd_packet(packet: SMDpacket, root_directory: &PathBuf) -> Self {
        assert!(root_directory.is_absolute());

        let data: Vec<u8> = packet.get_data().clone();
        return Self::new(String::from_utf8(data).unwrap(), root_directory).unwrap();
    }

    fn init_sync_directory(&self) -> () {
        let sync_directory: &PathBuf = self.get_sync_directory();
        let storage_directory: PathBuf = sync_directory.join(String::from("storage"));

        if !sync_directory.exists() {
            let _ = fs::create_dir(sync_directory);
        }

        if !storage_directory.exists() {
            let _ = fs::create_dir(storage_directory);
        }
    }

    pub fn get_sync_directory(&self) -> &PathBuf {
        &self.sync_directory
    }

    pub fn get_storage_directory(&self) -> PathBuf {
        self.get_sync_directory().join(String::from("storage"))
    }

    pub fn get_state(&self) -> Files {
        let state_path: PathBuf = self.get_sync_directory().join("smd_state.json");
        get_current_state(&self.get_storage_directory(), state_path).unwrap()
    }
}
