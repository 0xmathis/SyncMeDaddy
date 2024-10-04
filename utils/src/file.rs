use serde::{Serialize, Deserialize};
use std::fs;
use std::os::linux::fs::MetadataExt;
use std::path::PathBuf;

use crate::state::State;
use crate::hash;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct File {
    mtime: i64,
    size: u64,
    hash: [u8; 20],
    state: State,
}

impl File {
    pub fn new(filepath: PathBuf, state: State) -> Self {
        assert!(filepath.is_absolute());
        assert!(filepath.is_file());

        let metadata: fs::Metadata = fs::metadata(&filepath).unwrap();

        let mtime: i64 = metadata.st_mtime();
        let size: u64 = metadata.st_size();

        let hash: [u8; 20] = hash(&filepath);

        File::from_data(mtime, size, hash, state)
    }

    pub fn from_data(mtime: i64, size: u64, hash: [u8; 20], state: State) -> Self {
        Self {
            mtime,
            size,
            hash,
            state,
        }
    }

    pub fn mtime(&self) -> i64 {
        self.mtime
    }

   pub fn size(&self) -> u64 {
        self.size
    }

   pub fn hash(&self) -> [u8; 20] {
        self.hash
    }

   pub fn state(&self) -> &State {
       &self.state
   }

    pub fn set_state(&mut self, state: State) -> () {
        self.state = state;
    }
}
