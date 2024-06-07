use std::path::{Path, PathBuf};
use std::fs;
use std::io::Result;

use smd_protocol::smd_packet::SMDpacket;
use smd_protocol::smd_type::SMDtype;


pub struct User {
    username: String,
}

impl User {
    fn check_username(username: &String) -> bool {
        let username_path: &Path = Path::new(&username);

        if let Some(_) = username_path.parent() {
            return true;
        }

        false
    }

    pub fn new(username: String) -> Option<Self> {
        if !Self::check_username(&username) {
            return None;
        }

        Some(Self { username })
    }

    pub fn from_smd_packet(packet: SMDpacket) -> Option<Self> {
        match packet.get_type() {
            SMDtype::Connect => {
                let data: Vec<u8> = packet.get_data().clone();

                if data.is_ascii() {
                    return Self::new(String::from_utf8(data).unwrap())
                }
            }
            _ => return None,
        }

        None
    }

    fn init_sync_directory(&self, sync_directory: PathBuf) -> Result<()> {
        let sync_directory: PathBuf = self.get_sync_directory(sync_directory);
        fs::create_dir(sync_directory)
    }

    pub fn get_sync_directory(&self, sync_directory: PathBuf) -> PathBuf {
        return sync_directory.join(&self.username);
    }
}
