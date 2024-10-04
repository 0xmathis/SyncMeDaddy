use anyhow::{bail, Context, Result};
use log::{error, info};
use std::path::{Path, PathBuf};
use std::fs;

use smd_protocol::smd_packet::SMDpacket;
use utils::get_current_state;
use utils::files::Files;


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

    pub fn new(username: String, root_directory: &PathBuf) -> Result<Self> {
        assert!(root_directory.is_absolute());

        if !Self::check_username(&username) {
            bail!("Invalid username");
        }

        let sync_directory: PathBuf = root_directory.join(&username);
        let user: Self = Self { username, sync_directory };
        user.init_sync_directory();

        Ok(user)
    }

    pub fn from_smd_packet(packet: SMDpacket, root_directory: &PathBuf) -> Result<Self> {
        assert!(root_directory.is_absolute());

        let data: String = String::from_utf8(packet.data().clone()).context("Unable to deserialize SMDpacket to User")?;
        Self::new(data, root_directory).context("Unable to deserialize SMDpacket to User")
    }

    fn init_sync_directory(&self) -> () {
        let sync_directory: &PathBuf = self.sync_directory();
        let storage_directory: PathBuf = sync_directory.join(String::from("storage"));

        if !sync_directory.exists() {
            let _ = fs::create_dir(sync_directory);
        }

        if !storage_directory.exists() {
            let _ = fs::create_dir(storage_directory);
        }
    }

    pub fn username(&self) -> &String {
        &self.username
    }

    pub fn sync_directory(&self) -> &PathBuf {
        &self.sync_directory
    }

    pub fn state_path(&self) -> PathBuf {
        self.sync_directory().join("smd_state.json")
    }

    pub fn storage_directory(&self) -> PathBuf {
        self.sync_directory().join(String::from("storage"))
    }

    pub fn state(&self) -> Files {
        let state_path: PathBuf = self.sync_directory().join("smd_state.json");
        let stored_state: Files = Files::load_from_file(&state_path).unwrap();

        get_current_state(&self.storage_directory(), stored_state).unwrap()
    }

    pub fn store_state(&self) -> Result<()> {
        let state_path: PathBuf = self.state_path();

        match self.state().store_to_file(&state_path) {
            Ok(()) => info!("{}: State stored", self.username()),
            Err(e) => {
                error!("{}: Error storing state: {e}", self.username());
                bail!(e);
            },
        };

        Ok(())
    }
}
