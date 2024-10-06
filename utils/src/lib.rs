use anyhow::Result;
use path_absolutize::Absolutize;
use sha1::{Digest, Sha1};
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::{copy, Read};
use std::os::linux::fs::MetadataExt;
use std::path::PathBuf;
use std::rc::Rc;

use crate::file::File;
use crate::state::State;
use crate::files::Files;

pub mod data_transfer;
pub mod file;
pub mod state;
pub mod files;
pub mod update_answer;


pub fn to_valid_syncing_directory(sync_directory: PathBuf) -> PathBuf {
    let sync_directory: PathBuf = to_absolute_path(sync_directory);

    // if !sync_directory.is_dir() {
    //     panic!("Path provided ({}) is invalid", sync_directory.to_str().unwrap());
    // }

    sync_directory
}

pub fn to_absolute_path(path: PathBuf) -> PathBuf {
    path.absolutize().expect("Not sure if this can fail").to_path_buf()
}

pub fn to_relative_paths(paths: Vec<PathBuf>, root: &PathBuf) -> Vec<PathBuf> {
    assert!(root.is_absolute());

    paths
        .into_iter()
        .map(|path|
            to_absolute_path(path)
            .strip_prefix(root)
            .unwrap()
            .to_path_buf())
        .collect()
}

pub fn get_current_state(storage_directory: &PathBuf, stored_state: Files) -> Result<Files> {
    let mut paths: Vec<PathBuf> = tree_directory(storage_directory);
    paths = to_relative_paths(paths, storage_directory);

    let stored_state: &HashMap<Rc<PathBuf>, Rc<RefCell<File>>> = stored_state.data();
    let stored_filenames: HashSet<Rc<PathBuf>> = stored_state.keys().cloned().collect();
    let mut output: HashMap<Rc<PathBuf>, Rc<RefCell<File>>> = HashMap::new();

    // Start by checking already synchronised files
    for (filepath, file) in stored_state.into_iter() {
        assert_eq!(Rc::strong_count(&file), 1);

        if paths.contains(&filepath) {  // If the file still exists
            let absolute_path: PathBuf = storage_directory.join(filepath.to_path_buf());
            let metadata: fs::Metadata = fs::metadata(absolute_path).unwrap();

            let mtime: i64 = metadata.st_mtime();

            // Should not work
            if file.borrow().mtime() < mtime {  // If file has been modified
                file.borrow_mut().set_state(State::Edited);
            } else {
                file.borrow_mut().set_state(State::Unchanged);
            }
        } else {  // If file doesn't exist anymore
            file.borrow_mut().set_state(State::Deleted);
        }

        output.insert(Rc::clone(filepath), Rc::clone(file));
    }

    // Then checking new files
    for filepath in paths.into_iter() { 
        if stored_filenames.contains(&filepath) {
            continue;
        }

        let absolute_path: PathBuf = storage_directory.join(&filepath);
        let Ok(file) = File::new(absolute_path, State::Created) else {
            continue;
        };

        output.insert(Rc::new(filepath), Rc::new(RefCell::new(file)));
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
