use serde_json;
use serde::{Serialize, Deserialize};
use std::{collections::HashMap, fs};
use std::io::{Read, Result};
use std::path::PathBuf;

use crate::sha256sum;


#[derive(Debug, Serialize, Deserialize)]
pub struct JSON(HashMap<String, File>);

#[derive(Debug, Serialize, Deserialize)]
pub struct File {
    filepath: String,
    last_update: u64,
    sha256sum: String,
}

impl JSON {
    pub fn empty() -> Self {
        serde_json::from_str("{}").unwrap()
    }

    pub fn from_vec(files: Vec<PathBuf>) -> Self {
        let mut output: HashMap<String, File> = HashMap::new();
        println!("{}", files.len());

        for filepath in files.iter() {
            let file: File = File::new(filepath.clone(), 0, sha256sum(filepath.clone()).unwrap());

            output.insert(String::from(filepath.to_str().unwrap()), file);
        }

        JSON(output)
    }

    pub fn load_from_file(filepath: PathBuf) -> Result<Self> {
        if !filepath.exists() {
            return Ok(Self::empty());
        }

        let mut file = fs::File::open(filepath)?;
        let mut data = String::new();
        file.read_to_string(&mut data)?;

        let json: Self = serde_json::from_str(&data).expect("JSON was not well-formatted");

        Ok(json)
    }

    pub fn get_data(&self) -> &HashMap<String, File> {
        &self.0
    }
}

impl File {
    pub fn new(filepath: PathBuf, last_update: u64, sha256sum: String) -> Self {
        Self {
            filepath: filepath.to_str().unwrap().to_string(),
            last_update,
            sha256sum,
        }
    }
}
