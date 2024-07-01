use std::collections::HashMap;
use std::fs::{self, read_dir, ReadDir};
use std::io::{self, Result};
use std::os::linux::fs::MetadataExt;
use std::path::PathBuf;
use std::str::FromStr;
use my_json::{File, FileState, JSON};
use path_absolutize::Absolutize;
use sha1::{Digest, Sha1};
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

pub fn get_current_state(paths: Vec<PathBuf>, root_directory: &PathBuf, mut stored_state: JSON) -> Result<JSON> {
    let stored_state: &mut HashMap<PathBuf, File> = stored_state.get_mut_data();
    let mut output: HashMap<PathBuf, File> = HashMap::new();

    // Start by checking already synchronised files
    for (filepath, file) in stored_state.iter_mut() {
        print!("File {filepath:?} ");

        if paths.contains(filepath) {  // If the file still exists
            let absolute_path: PathBuf = root_directory.join(filepath);
            let metadata: fs::Metadata = fs::metadata(&absolute_path).unwrap();

            let mtime: i64 = metadata.st_mtime();

            if file.get_mtime() <  mtime {  // If file has been modified
                println!("edited");
                file.set_state(FileState::Edited);
            } else {
                println!("unchanged");
                file.set_state(FileState::Unchanged);
            }
        } else {  // If file doesn't exist anymore
            println!("deleted");
            file.set_state(FileState::Deleted);
        }

        output.insert(filepath.to_path_buf(), file.to_owned());
    }

    // Then checking new files
    for filepath in paths.iter() { 
        if stored_state.contains_key(filepath) {
            continue;
        }

        println!("File {filepath:?} created");
        let absolute_path: PathBuf = root_directory.join(filepath);
        let file: File = File::new(&absolute_path, FileState::Created);
        output.insert(filepath.to_path_buf(), file);
    }

    Ok(JSON::from_map(output))
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
