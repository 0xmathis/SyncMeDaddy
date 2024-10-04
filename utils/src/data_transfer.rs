use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::{from_slice, json};
use std::fs;
use std::io::Write;
use std::path::PathBuf;

use crate::file::File;


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DataTransfer {
    filename: PathBuf,
    file: File,
    data: Vec<u8>,
}

impl DataTransfer {
    pub fn new(filename: PathBuf, file: File, data: Vec<u8>) -> Self {
        Self {
            filename,
            file,
            data,
        }
    }

    pub fn from_vec(data: &Vec<u8>) -> Self {
        from_slice(data).unwrap()
    }

    pub fn to_vec(&self) -> Vec<u8> {
        Vec::from(json!(self).to_string())
    }

    pub fn filename(&self) -> &PathBuf {
        &self.filename
    }

    pub fn file(&self) -> &File {
        &self.file
    }

    pub fn data(&self) -> &Vec<u8> {
        &self.data
    }

    pub fn store(self, root_directory: &PathBuf) -> Result<()> {
        assert!(root_directory.is_absolute());

        let filename: &PathBuf = self.filename();
        let filepath: PathBuf = root_directory.join(filename);
        let file_parents: PathBuf = root_directory.join(filename.parent().unwrap());
        fs::create_dir_all(file_parents)?;
        let mut file_writer = fs::File::create(filepath)?;
        file_writer.write_all(self.data())?;
        file_writer.sync_all()?;

        Ok(())
    }
}
