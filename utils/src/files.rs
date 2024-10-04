use anyhow::Result;
use serde::{Serialize, Deserialize};
use serde_json::{from_str, json};
use std::collections::HashMap;
use std::fs;
use std::io::{Read, Write};
use std::path::PathBuf;

use crate::file::File;
use crate::state::State;


#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Files(HashMap<PathBuf, File>);

impl Files {
    pub fn empty() -> Self {
        from_str("{}").unwrap()
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

    pub fn store_to_file(mut self, filepath: &PathBuf) -> Result<()> {
        for (_, file) in self.data_mut().iter_mut() {
            file.set_state(State::Unchanged);
        }

        let json_string = serde_json::to_string(&self).unwrap();
        let mut file = fs::File::create(filepath).unwrap();

        Ok(file.write_all(json_string.as_bytes())?)
    }

    pub fn data(&self) -> &HashMap<PathBuf, File> {
        &self.0
    }

    pub fn data_mut(&mut self) -> &mut HashMap<PathBuf, File> {
        &mut self.0
    }
}
