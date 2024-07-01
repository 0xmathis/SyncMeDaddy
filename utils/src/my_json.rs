use serde::{Serialize, Deserialize};
use serde_json;
use std::collections::HashMap;
use std::collections::HashSet;
use std::fs;
use std::io::Write;
use std::io::{self, Read, Result};
use std::os::linux::fs::MetadataExt;
use std::path::PathBuf;
use sha1::{Sha1, Digest};

use crate::to_absolute_path;


// #[derive(Debug, Serialize, Deserialize)]
// pub struct UpdateAnswer {
//     to_upload: JSON,
//     to_download: JSON,
// }

#[derive(Debug, Serialize, Deserialize)]
pub struct JSON(HashMap<PathBuf, File>);

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct File {
    mtime: i64,
    size: u64,
    sha1: [u8; 20],
    state: FileState,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum FileState {
    Unchanged,
    Created,
    Edited,
    Deleted,
}

impl JSON {
    // pub fn diff(self, other: Self) -> Self {
    //     let self_data = self.get_data();
    //     let other_data = other.get_data().to_owned();
    //     let mut files: HashSet<&String> = HashSet::new();
    //     files.extend(self_data.keys());
    //     files.extend(other_data.keys());
    //     let mut output: HashMap<String, File> = HashMap::new();
        
    //     for filepath in files {
    //         let self_contains: bool = self_data.contains_key(filepath);
    //         let other_contains: bool = other_data.contains_key(filepath);

    //         if self_contains && other_contains {
    //         } else if self_contains {
    //         } else if other_contains {
    //             let mut file: File = other_data.get(filepath).unwrap().to_owned();
    //             file.set_state(FileState::Edited);
    //         }
    //     }

    //     Self::empty()
    // }

    pub fn empty() -> Self {
        serde_json::from_str("{}").unwrap()
    }

    pub fn from_map(map: HashMap<PathBuf, File>) -> Self {
        Self(map)
    }

    // pub fn from_paths(paths: Vec<PathBuf>, root_directory: &PathBuf) -> Self {
    //     let mut output: HashMap<PathBuf, File> = HashMap::new();

    //     for filepath in paths.iter() { 
    //         let file: File = File::new(&filepath, 0);
    //         let filepath_stripped: PathBuf = to_absolute_path(filepath).strip_prefix(&root_directory).unwrap().to_path_buf();

    //         output.insert(filepath_stripped, file);
    //     }

    //     Self(output)
    // }

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

    pub fn store_to_file(&self, filepath: &PathBuf) -> Result<()> {
        let json_string = serde_json::to_string(self).unwrap();
        let mut file = fs::File::create(filepath).unwrap();

        file.write_all(json_string.as_bytes())
    }

    pub fn get_mut_data(&mut self) -> &mut HashMap<PathBuf, File> {
        &mut self.0
    }

    pub fn get_data(&self) -> &HashMap<PathBuf, File> {
        &self.0
    }
}

impl File {
    pub fn new(filepath: &PathBuf, state: FileState) -> Self {
        let metadata: fs::Metadata = fs::metadata(filepath).unwrap();

        let mtime: i64 = metadata.st_mtime();
        let size: u64 = metadata.st_size();

        let mut file: fs::File = fs::File::open(&filepath).unwrap();
        let mut hasher: Sha1 = Sha1::new();
        io::copy(&mut file, &mut hasher).unwrap();
        let sha1: [u8; 20] = hasher.finalize().into();

        Self {
            mtime,
            size,
            sha1,
            state,
        }
    }

    pub fn get_mtime(&self) -> i64 {
        self.mtime
    }

   pub fn get_size(&self) -> u64 {
        self.size
    }

   pub fn get_sha1(&self) -> &[u8; 20] {
        &self.sha1
    }

    pub fn set_state(&mut self, state: FileState) -> () {
        self.state = state;
    }
}
