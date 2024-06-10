use std::fs::{read_dir, ReadDir};
use std::io::Result;
use std::path::{Path, PathBuf};
use my_json::JSON;
use path_absolutize::Absolutize;
use std::time::{SystemTime, UNIX_EPOCH};
use sha256::try_digest;

pub mod my_json;


pub fn to_valid_syncing_directory(sync_directory: String) -> Result<PathBuf> {
    let sync_directory: PathBuf = Path::new(&sync_directory).absolutize()?.to_path_buf();

    if !sync_directory.is_dir() {
        panic!("Path provided ({}) is invalid", sync_directory.to_str().unwrap());
    }

    Ok(sync_directory)
}

pub fn get_current_state(sync_directory: PathBuf) -> Result<JSON> {
    let files: Vec<PathBuf> = tree_directory(sync_directory)?;
    return Ok(JSON::from_paths(files));
}

pub fn tree_directory(directory: PathBuf) -> Result<Vec<PathBuf>> {
    let mut output: Vec<PathBuf> = Vec::new();

    let paths: ReadDir = read_dir(directory)?;

    for path in paths {
        let path: PathBuf = path?.path();

        if path.is_file() {
            output.push(path);
        } else if path.is_dir() {
            output.append(&mut tree_directory(path)?);
        }
    }

    Ok(output)
}

pub fn time_since_epoch() -> u64 {
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(n) => n.as_secs(),
        Err(_) => 0,
    }
}

pub fn last_modified(filepath: &PathBuf) -> Result<u64> {
    Ok(filepath.metadata()?.modified()?.duration_since(UNIX_EPOCH).unwrap().as_secs())
}

pub fn sha256sum(filepath: PathBuf) -> Result<String> {
    try_digest(filepath)
}
