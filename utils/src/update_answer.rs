use serde::{Serialize, Deserialize};
use serde_json::json;

use crate::files::Files;


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UpdateAnswer {
    to_upload: Files,
    to_download: Files,
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
