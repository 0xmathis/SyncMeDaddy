use serde_json;
use serde::{Serialize, Deserialize};
use std::{collections::HashMap, fs};
use std::io::{Read, Result};
use std::path::PathBuf;

use crate::last_modified;


#[derive(Debug, Serialize, Deserialize)]
pub struct JSON(HashMap<String, File>);

#[derive(Debug, Serialize, Deserialize)]
pub struct File {
    filepath: String,
    last_update: u64,
}

impl JSON {
    pub fn empty() -> Self {
        serde_json::from_str("{}").unwrap()
    }

    pub fn from_paths(paths: Vec<PathBuf>) -> Self {
        let mut output: HashMap<String, File> = HashMap::new();
        println!("{}", paths.len());

        for filepath in paths.iter() {
            if let Ok(last_modified) = last_modified(filepath) {
                let file: File = File::new(filepath, last_modified);

                output.insert(String::from(filepath.to_str().unwrap()), file);
            }
        }

        JSON(output)
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
            filepath: filepath.to_str().unwrap().to_string(),
            last_update,
        }
    }
}
