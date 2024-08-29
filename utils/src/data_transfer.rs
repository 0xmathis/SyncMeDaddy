use anyhow::Result;
use serde::{Serialize, Deserialize};
use serde_json::json;
use std::fs::{self, create_dir_all};
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
        serde_json::from_slice(data).unwrap()
    }

    pub fn to_vec(&self) -> Vec<u8> {
        Vec::from(json!(self).to_string())
    }

    pub fn get_filename(&self) -> &PathBuf {
        &self.filename
    }

    pub fn get_file(&self) -> &File {
        &self.file
    }

    pub fn get_data(&self) -> &Vec<u8> {
        &self.data
    }

    pub fn store(self, root_directory: &PathBuf) -> Result<()> {
        assert!(root_directory.is_absolute());

        let filename: &PathBuf = self.get_filename();
        let filepath: PathBuf = root_directory.join(filename);
        let file_parents: PathBuf = root_directory.join(filename.parent().unwrap());
        create_dir_all(file_parents)?;
        let mut file_writer = fs::File::create(filepath)?;
        file_writer.write_all(self.get_data())?;
        file_writer.sync_all()?;

        Ok(())
    }
}
