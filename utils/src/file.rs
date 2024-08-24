use serde::{Serialize, Deserialize};
use std::fs;
use std::os::linux::fs::MetadataExt;
use std::path::PathBuf;

use crate::file_state::FileState;
use crate::hash;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct File {
    mtime: i64,
    size: u64,
    hash: [u8; 20],
    state: FileState,
}

impl File {
    pub fn new(filepath: PathBuf, state: FileState) -> Self {
        let metadata: fs::Metadata = fs::metadata(&filepath).unwrap();

        let mtime: i64 = metadata.st_mtime();
        let size: u64 = metadata.st_size();

        let hash: [u8; 20] = hash(&filepath);

        Self {
            mtime,
            size,
            hash,
            state,
        }
    }

    pub fn get_mtime(&self) -> i64 {
        self.mtime
    }

   pub fn get_size(&self) -> u64 {
        self.size
    }

   pub fn get_hash(&self) -> [u8; 20] {
        self.hash
    }

    pub fn set_state(&mut self, state: FileState) -> () {
        self.state = state;
    }
}
