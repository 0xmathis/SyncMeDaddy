use serde::{Serialize, Deserialize};
use serde_json::json;

use crate::files::Files;


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UpdateAnswer {
    server_todo: Files,
    client_todo: Files,
}

impl UpdateAnswer {
    pub fn from_vec(data: &Vec<u8>) -> Self {
        serde_json::from_slice(data).unwrap()
    }

    pub fn from_json(server_todo: Files, client_todo: Files) -> UpdateAnswer {
        UpdateAnswer {
            server_todo,
            client_todo,
        }
    }

    pub fn to_vec(&self) -> Vec<u8> {
        Vec::from(json!(self).to_string())
    }

    pub fn data(self) -> (Files, Files) {
        (self.server_todo, self.client_todo)
    }
}
