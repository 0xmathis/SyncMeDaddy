use std::{path::PathBuf, str::FromStr};

use utils::my_json::{Files};
use utils::get_current_state;

fn main() {
    let root: PathBuf = PathBuf::from_str("/tmp/smd/client").unwrap();
    let storage_directory: PathBuf = root.join("storage");
    let state: PathBuf = root.join("smd_state.json");

    let stored_state: Files = Files::load_from_file(&state).unwrap();
    let current_state: Files = get_current_state(&storage_directory, stored_state).unwrap();

    // let storage: PathBuf = PathBuf::from_str("/tmp/smd/client/smd_state.json").unwrap();
    // current_state.store_to_file(&storage).unwrap();
}
