use std::path::{Path, PathBuf};

use path_absolutize::Absolutize;

pub fn to_valid_syncing_directory(sync_directory: String) -> PathBuf {
    let sync_directory: PathBuf = Path::new(&sync_directory).absolutize().unwrap().to_path_buf();

    if !sync_directory.is_dir() {
        panic!("Path provided ({}) is invalid", sync_directory.to_str().unwrap());
    }

    sync_directory
}
