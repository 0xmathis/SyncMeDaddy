use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum FileState {
    Unchanged,
    Created,
    Edited,
    Deleted,
}
