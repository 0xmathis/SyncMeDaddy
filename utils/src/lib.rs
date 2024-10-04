use anyhow::Result;
use path_absolutize::Absolutize;
use sha1::{Digest, Sha1};
use std::collections::HashMap;
use std::fs;
use std::io::{copy, Read};
use std::os::linux::fs::MetadataExt;
use std::path::PathBuf;
use std::str::FromStr;

use crate::file::File;
use crate::state::State;
use crate::files::Files;

pub mod data_transfer;
pub mod file;
pub mod state;
pub mod files;
pub mod update_answer;


pub fn to_valid_syncing_directory(sync_directory: String) -> PathBuf {
    let sync_directory: PathBuf = to_absolute_path(PathBuf::from_str(&sync_directory).unwrap());

    // if !sync_directory.is_dir() {
    //     panic!("Path provided ({}) is invalid", sync_directory.to_str().unwrap());
    // }

    sync_directory
}

pub fn to_absolute_path(path: PathBuf) -> PathBuf {
    path.absolutize().unwrap().to_path_buf()
}

pub fn to_relative_paths(paths: Vec<PathBuf>, root: &PathBuf) -> Vec<PathBuf> {
    paths
        .into_iter()
        .map(|path|
            to_absolute_path(path)
            .strip_prefix(root)
            .unwrap()
            .to_path_buf())
        .collect()
}

pub fn get_current_state(storage_directory: &PathBuf, mut stored_state: Files) -> Result<Files> {
    let mut paths: Vec<PathBuf> = tree_directory(storage_directory);
    paths = to_relative_paths(paths, storage_directory);

    let stored_state: &mut HashMap<PathBuf, File> = stored_state.data_mut();
    let mut output: HashMap<PathBuf, File> = HashMap::new();

    // Start by checking already synchronised files
    for (filepath, file) in stored_state.iter_mut() {
        if paths.contains(filepath) {  // If the file still exists
            let absolute_path: PathBuf = storage_directory.join(filepath);
            let metadata: fs::Metadata = fs::metadata(absolute_path).unwrap();

            let mtime: i64 = metadata.st_mtime();

            if file.mtime() < mtime {  // If file has been modified
                file.set_state(State::Edited);
            } else {
                file.set_state(State::Unchanged);
            }
        } else {  // If file doesn't exist anymore
            file.set_state(State::Deleted);
        }

        output.insert(filepath.to_path_buf(), file.clone());
    }

    // Then checking new files
    for filepath in paths.into_iter() { 
        if stored_state.contains_key(&filepath) {
            continue;
        }

        let absolute_path: PathBuf = storage_directory.join(&filepath);
        let file: File = File::new(absolute_path, State::Created);
        output.insert(filepath, file);
    }

    Ok(Files::from_map(output))
}

pub fn tree_directory(directory: &PathBuf) -> Vec<PathBuf> {
    let Ok(paths) = fs::read_dir(directory) else {
        return Vec::new();
    };

    let mut output: Vec<PathBuf> = Vec::new();

    for path in paths {
        let Ok(path) = path else {
            continue;
        };

        let path: PathBuf = path.path();

        if path.is_file() {
            output.push(path);
        } else if path.is_dir() {
            output.append(&mut tree_directory(&path));
        }
    }

    output
}

pub fn hash(filepath: &PathBuf) -> [u8; 20] {
    let mut file: fs::File = fs::File::open(filepath).unwrap();
    let mut hasher: Sha1 = Sha1::new();
    copy(&mut file, &mut hasher).unwrap();
    hasher.finalize().into()
}

pub fn read_file(filepath: PathBuf, filesize: usize) -> Result<Vec<u8>> {
    let mut buffer: Vec<u8> = Vec::new();
    buffer.resize(filesize, 0);
    let mut file_reader: fs::File = fs::File::open(filepath)?;
    file_reader.read_exact(&mut buffer)?;

    Ok(buffer)
}
