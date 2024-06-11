use std::fs::{read_dir, ReadDir};
use std::io::Result;
use std::path::PathBuf;
use std::str::FromStr;
use my_json::JSON;
use path_absolutize::Absolutize;
use std::time::{SystemTime, UNIX_EPOCH};

pub mod my_json;


pub fn to_valid_syncing_directory(sync_directory: String) -> Result<PathBuf> {
    let sync_directory: PathBuf = to_absolute_path(&PathBuf::from_str(&sync_directory).unwrap());

    if !sync_directory.is_dir() {
        panic!("Path provided ({}) is invalid", sync_directory.to_str().unwrap());
    }

    Ok(sync_directory)
}

pub fn to_absolute_path(path: &PathBuf) -> PathBuf {
    path.absolutize().unwrap().to_path_buf()
}

pub fn get_current_state(root_directory: &PathBuf) -> Result<JSON> {
    let mut files: Vec<PathBuf> = tree_directory(root_directory)?;
    
    if let Ok(index) = files.binary_search(&root_directory.join(".smd_state")) {
        files.remove(index);
    }

    return Ok(JSON::from_paths(files, root_directory));
}

pub fn tree_directory(directory: &PathBuf) -> Result<Vec<PathBuf>> {
    let mut output: Vec<PathBuf> = Vec::new();

    let paths: ReadDir = read_dir(directory)?;

    for path in paths {
        let path: PathBuf = path?.path();

        if path.is_file() {
            output.push(path);
        } else if path.is_dir() {
            output.append(&mut tree_directory(&path)?);
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
