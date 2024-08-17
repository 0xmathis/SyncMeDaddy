use serde::{Serialize, Deserialize};
use serde_json;
use serde_json::json;
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::io::{self, Read, Result};
use std::os::linux::fs::MetadataExt;
use std::path::PathBuf;
use sha1::{Sha1, Digest};


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UpdateAnswer {
    to_upload: Files,
    to_download: Files,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DataTransfer {
    filename: PathBuf,
    file: File,
    data: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Files(HashMap<PathBuf, File>);

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct File {
    mtime: i64,
    size: u64,
    hash: [u8; 20],
    state: FileState,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum FileState {
    Unchanged,
    Created,
    Edited,
    Deleted,
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
        let filepath: PathBuf = self.get_filename().join(root_directory);
        let mut file_writer = fs::File::create(filepath)?;
        file_writer.write_all(self.get_data())?;
        file_writer.sync_all()?;

        Ok(())
    }
}

impl UpdateAnswer {
    pub fn from_vec(data: &Vec<u8>) -> Self {
        serde_json::from_slice(data).unwrap()
    }

    pub fn from_json(to_upload: Files, to_download: Files) -> UpdateAnswer {
        UpdateAnswer {
            to_upload,
            to_download,
        }
    }

    pub fn to_vec(&self) -> Vec<u8> {
        Vec::from(json!(self).to_string())
    }

    pub fn get_data(self) -> (Files, Files) {
        (self.to_upload, self.to_download)
    }
}

impl Files {
    pub fn empty() -> Self {
        serde_json::from_str("{}").unwrap()
    }

    pub fn from_map(map: HashMap<PathBuf, File>) -> Self {
        Self(map)
    }

    pub fn from_str(data: &String) -> Self {
        serde_json::from_str(data).unwrap()
    }

    pub fn from_vec(data: &Vec<u8>) -> Self {
        serde_json::from_slice(data).unwrap()
    }

    pub fn load_from_file(filepath: &PathBuf) -> Result<Self> {
        if !filepath.exists() || !filepath.is_file() {
            return Ok(Self::empty());
        }

        let mut file: fs::File = fs::File::open(filepath)?;
        let mut data: String = String::new();
        file.read_to_string(&mut data)?;

        let json: Self = Self::from_str(&data);

        Ok(json)
    }

    pub fn to_vec(&self) -> Vec<u8> {
        Vec::from(json!(self).to_string())
    }

    pub fn store_to_file(&self, filepath: &PathBuf) -> Result<()> {
        let json_string = serde_json::to_string(self).unwrap();
        let mut file = fs::File::create(filepath).unwrap();

        file.write_all(json_string.as_bytes())
    }

    pub fn get_data(self) -> HashMap<PathBuf, File> {
        self.0
    }
}

impl File {
    pub fn new(filepath: PathBuf, state: FileState) -> Self {
        let metadata: fs::Metadata = fs::metadata(&filepath).unwrap();

        let mtime: i64 = metadata.st_mtime();
        let size: u64 = metadata.st_size();

        let hash: [u8; 20] = Self::hash(&filepath);

        Self {
            mtime,
            size,
            hash,
            state,
        }
    }

    fn hash(filepath: &PathBuf) -> [u8; 20] {
        let mut file: fs::File = fs::File::open(&filepath).unwrap();
        let mut hasher: Sha1 = Sha1::new();
        io::copy(&mut file, &mut hasher).unwrap();
        return hasher.finalize().into();
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
