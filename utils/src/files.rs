use anyhow::Result;
use serde::{Serialize, Deserialize};
use serde_json::json;
use std::collections::HashMap;
use std::fs;
use std::io::{Read, Write};
use std::path::PathBuf;

use crate::file::File;


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Files(HashMap<PathBuf, File>);

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

        Ok(file.write_all(json_string.as_bytes())?)
    }

    pub fn get_data(self) -> HashMap<PathBuf, File> {
        self.0
    }
}
