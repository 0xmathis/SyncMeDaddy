use std::{io::Result, path::{Path, PathBuf}};

use path_absolutize::Absolutize;

pub fn to_valid_syncing_directory(sync_directory: String) -> Result<PathBuf> {
    let sync_directory: PathBuf = Path::new(&sync_directory).absolutize()?.to_path_buf();

    if !sync_directory.is_dir() {
        panic!("Path provided ({}) is invalid", sync_directory.to_str().unwrap());
    }

    Ok(sync_directory)
}
