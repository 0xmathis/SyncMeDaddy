use my_json::{File, FileState, Files};
use path_absolutize::Absolutize;
use sha1::{Digest, Sha1};
use std::collections::HashMap;
use std::fs::{self, read_dir, ReadDir};
use std::io::{self, Result};
use std::os::linux::fs::MetadataExt;
use std::path::PathBuf;
use std::str::FromStr;
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

pub fn get_current_state(storage_directory: &PathBuf, stored_state: Files) -> Result<Files> {
    let mut paths: Vec<PathBuf> = tree_directory(storage_directory).unwrap();
    paths = paths.iter().map(|path| to_absolute_path(path).strip_prefix(storage_directory).unwrap().to_path_buf()).collect();

    let mut stored_state: HashMap<PathBuf, File> = stored_state.get_data();
    let mut output: HashMap<PathBuf, File> = HashMap::new();

    // Start by checking already synchronised files
    for (filepath, file) in stored_state.iter_mut() {
        if paths.contains(filepath) {  // If the file still exists
            let absolute_path: PathBuf = storage_directory.join(filepath);
            let metadata: fs::Metadata = fs::metadata(&absolute_path).unwrap();

            let mtime: i64 = metadata.st_mtime();

            if file.get_mtime() < mtime {  // If file has been modified
                file.set_state(FileState::Edited);
            } else {
                file.set_state(FileState::Unchanged);
            }
        } else {  // If file doesn't exist anymore
            file.set_state(FileState::Deleted);
        }

        output.insert(filepath.to_path_buf(), file.clone());
    }

    // Then checking new files
    for filepath in paths.iter() { 
        if stored_state.contains_key(filepath) {
            continue;
        }

        let absolute_path: PathBuf = storage_directory.join(filepath);
        let file: File = File::new(&absolute_path, FileState::Created);
        output.insert(filepath.to_path_buf(), file);
    }

    Ok(Files::from_map(output))
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

pub fn sha1(filepath: &PathBuf) -> [u8; 20] {
    let mut file: fs::File = fs::File::open(filepath).unwrap();
    let mut hasher: Sha1 = Sha1::new();
    io::copy(&mut file, &mut hasher).unwrap();
    hasher.finalize().into()
}
