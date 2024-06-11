use serde_json;
use serde::{Serialize, Deserialize};
use std::collections::HashSet;
use std::{collections::HashMap, fs};
use std::io::{Read, Result};
use std::path::PathBuf;

use crate::{last_modified, to_absolute_path};


#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateAnswer {
    to_upload: JSON,
    to_download: JSON,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JSON(HashMap<String, File>);

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct File {
    filepath: PathBuf,
    last_update: u64,
    state: Option<FileState>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum FileState {
    Edited,
    Deleted,
    Stored,
}

impl JSON {
    pub fn diff(self, other: Self) -> Self {
        let self_data = self.get_data();
        let other_data = other.get_data().to_owned();
        let mut files: HashSet<&String> = HashSet::new();
        files.extend(self_data.keys());
        files.extend(other_data.keys());
        let mut output: HashMap<String, File> = HashMap::new();
        
        for filepath in files {
            let self_contains: bool = self_data.contains_key(filepath);
            let other_contains: bool = other_data.contains_key(filepath);

            if self_contains && other_contains {
            } else if self_contains {
            } else if other_contains {
                let mut file: File = other_data.get(filepath).unwrap().to_owned();
                file.set_state(FileState::Edited);
            }
        }

        Self::empty()
    }

    pub fn empty() -> Self {
        serde_json::from_str("{}").unwrap()
    }

    pub fn from_map(map: HashMap<String, File>) -> Self {
        Self(map)
    }

    pub fn from_paths(paths: Vec<PathBuf>, root_directory: &PathBuf) -> Self {
        let mut output: HashMap<String, File> = HashMap::new();

        for filepath in paths.iter() { 
            if let Ok(last_modified) = last_modified(filepath) {
                let filepath: PathBuf = to_absolute_path(filepath).strip_prefix(&root_directory).unwrap().to_path_buf();
                let file: File = File::new(&filepath, last_modified);

                output.insert(String::from(filepath.to_str().unwrap()), file);
            }
        }

        Self::from_map(output)
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

    pub fn get_data(&self) -> &HashMap<String, File> {
        &self.0
    }
}

impl File {
    pub fn new(filepath: &PathBuf, last_update: u64) -> Self {
        Self {
            filepath: filepath.to_owned(),
            last_update,
            state: None,
        }
    }

    pub fn set_state(&mut self, state: FileState) -> () {
        self.state = Some(state);
    }

    pub fn del_state(&mut self) -> () {
        self.state = None;
    }
}
