use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum State {
    Unchanged,
    Created,
    Edited,
    Deleted,
}
