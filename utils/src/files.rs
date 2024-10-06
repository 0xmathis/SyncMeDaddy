use anyhow::{Context, Result};
use serde::{Serialize, Deserialize};
use serde_json::{from_str, json};
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::rc::Rc;

use crate::file::File;
use crate::state::State;


#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Files(HashMap<Rc<PathBuf>, Rc<RefCell<File>>>);

impl Files {
    pub fn empty() -> Self {
        from_str("{}").expect("I swear this is valid data")
    }

    pub fn from_map(map: HashMap<Rc<PathBuf>, Rc<RefCell<File>>>) -> Self {
        Self(map)
    }

    pub fn from_str(data: &String) -> Result<Self> {
        serde_json::from_str(data).context("Unable to deserialize to Files from String")
    }

    pub fn from_vec(data: &Vec<u8>) -> Result<Self> {
        serde_json::from_slice(data).context("Unable to deserialize to Files from Vec<u8>")
    }

    pub fn load_from_file(filepath: &PathBuf) -> Result<Self> {
        if !filepath.exists() || !filepath.is_file() {
            return Ok(Self::empty());
        }

        let mut file: fs::File = fs::File::open(filepath)?;
        let mut data: String = String::new();
        file.read_to_string(&mut data).context(format!("Fail to read file {filepath:?}"))?;

        let json: Self = Self::from_str(&data)?;

        Ok(json)
    }

    pub fn to_vec(&self) -> Vec<u8> {
        Vec::from(json!(self).to_string())
    }

    pub fn store_to_file(self, filepath: &PathBuf) -> Result<()> {
        assert!(
            self.data()
            .iter()
            .all(
                |(_, file)| Rc::strong_count(file) == 1
            ));

        let _ = self.data()
            .iter()
            .map(|(_, file)| {
                file.borrow_mut().set_state(State::Unchanged);
            })
        .collect::<Vec<_>>();

        let json_string = serde_json::to_string(&self).expect("Should not panic");
        let mut file = fs::File::create(filepath).context("Unable to open file")?;

        Ok(file.write_all(json_string.as_bytes())?)
    }

    pub fn data(&self) -> &HashMap<Rc<PathBuf>, Rc<RefCell<File>>> {
        &self.0
    }

    pub fn diff(server_data: Self, client_data: Self) -> (Self, Self) {
        /*
         * server_todo means files the server needs to update
         * client_todo means files the client needs to update
         */

        let mut server_todo: HashMap<Rc<PathBuf>, Rc<RefCell<File>>> = HashMap::new();
        let mut client_todo: HashMap<Rc<PathBuf>, Rc<RefCell<File>>> = HashMap::new();
        let server_data: &HashMap<Rc<PathBuf>, Rc<RefCell<File>>> = server_data.data();
        let client_data: &HashMap<Rc<PathBuf>, Rc<RefCell<File>>> = client_data.data();

        let mut filenames: HashSet<Rc<PathBuf>> = HashSet::new();
        let server_filenames: HashSet<Rc<PathBuf>> = server_data.keys().cloned().collect();
        let client_filenames: HashSet<Rc<PathBuf>> = client_data.keys().cloned().collect();
        filenames.extend(server_filenames);
        filenames.extend(client_filenames);

        for filename in filenames.into_iter() {
            let server_contains: bool = server_data.contains_key(&filename);
            let client_contains: bool = client_data.contains_key(&filename);

            if server_contains && client_contains { // If both have the file stored
                let server_file: &Rc<RefCell<File>> = server_data.get(&filename).expect("Can't panic");
                let server_borrow = server_file.borrow();
                let client_file: &Rc<RefCell<File>> = client_data.get(&filename).expect("Can't panic");
                let client_borrow = client_file.borrow();

                match (server_borrow.state(), client_borrow.state()) {
                    (State::Unchanged, State::Unchanged) => {},
                    (State::Unchanged, _) => {
                        server_todo.insert(filename, Rc::clone(client_file));
                    },
                    (_, State::Unchanged) => {
                        client_todo.insert(filename, Rc::clone(server_file));
                    },
                    (State::Created, State::Created) |
                        (State::Created, State::Edited) |
                        (State::Edited, State::Created) |
                        (State::Edited, State::Edited) => {
                            if server_borrow.hash() != client_borrow.hash() {
                                // If files are different, we keep the one modified last
                                if server_borrow.mtime() < client_borrow.mtime() { // Last version on client
                                    server_todo.insert(filename, client_file.clone());
                                } else { // Last version on server
                                    client_todo.insert(filename, server_file.clone());
                                }
                            }
                        },
                        (State::Deleted, State::Deleted) => todo!(),
                        (State::Edited, State::Deleted) => todo!(),
                        (State::Deleted, State::Edited) => todo!(),
                        (State::Deleted, State::Created) => todo!(),
                        (State::Created, State::Deleted) => todo!(),
                }
            } else if server_contains && !client_contains { // If only the server have the file stored
                let server_file: &Rc<RefCell<File>> = server_data.get(&filename).expect("Can't panic");

                if State::Unchanged.ne(server_file.borrow().state()) {
                    client_todo.insert(filename, Rc::clone(server_file));
                }
            } else if client_contains && !server_contains { // If only the client have the file stored
                let client_file: &Rc<RefCell<File>> = client_data.get(&filename).expect("Can't panic");

                if State::Unchanged.ne(client_file.borrow().state()) {
                    server_todo.insert(filename, Rc::clone(client_file));
                }
            }
        }

        (Self::from_map(server_todo), Self::from_map(client_todo))
    }
}

#[cfg(test)]
mod server {
    use super::*;

    #[test]
    fn test_state_diff_unchanged_unchanged() {
        let mut server_input: HashMap<Rc<PathBuf>, Rc<RefCell<File>>> = HashMap::new();
        let mut client_input: HashMap<Rc<PathBuf>, Rc<RefCell<File>>> = HashMap::new();
        let server_output: HashMap<Rc<PathBuf>, Rc<RefCell<File>>> = HashMap::new();
        let client_output: HashMap<Rc<PathBuf>, Rc<RefCell<File>>> = HashMap::new();

        server_input.insert(Rc::new(PathBuf::from("file")), Rc::new(RefCell::new(File::from_data(1, 2, [0; 20], State::Unchanged))));
        client_input.insert(Rc::new(PathBuf::from("file")), Rc::new(RefCell::new(File::from_data(2, 2, [0; 20], State::Unchanged))));

        let server_data: Files = Files::from_map(server_input);
        let client_data: Files = Files::from_map(client_input);
        let server_todo: Files = Files::from_map(server_output);
        let client_todo: Files = Files::from_map(client_output);
        let result: (Files, Files) = Files::diff(server_data, client_data);

        assert_eq!(result, (server_todo, client_todo));
    }

    #[test]
    fn test_state_diff_unchanged_created() {
        let mut server_input: HashMap<Rc<PathBuf>, Rc<RefCell<File>>> = HashMap::new();
        let mut client_input: HashMap<Rc<PathBuf>, Rc<RefCell<File>>> = HashMap::new();
        let mut server_output: HashMap<Rc<PathBuf>, Rc<RefCell<File>>> = HashMap::new();
        let client_output: HashMap<Rc<PathBuf>, Rc<RefCell<File>>> = HashMap::new();

        server_input.insert(Rc::new(PathBuf::from("file")), Rc::new(RefCell::new(File::from_data(1, 2, [0; 20], State::Unchanged))));
        client_input.insert(Rc::new(PathBuf::from("file")), Rc::new(RefCell::new(File::from_data(2, 3, [1; 20], State::Created))));
        server_output.insert(Rc::new(PathBuf::from("file")), Rc::new(RefCell::new(File::from_data(2, 3, [1; 20], State::Created))));

        let server_data: Files = Files::from_map(server_input);
        let client_data: Files = Files::from_map(client_input);
        let server_todo: Files = Files::from_map(server_output);
        let client_todo: Files = Files::from_map(client_output);
        let result: (Files, Files) = Files::diff(server_data, client_data);

        assert_eq!(result, (server_todo, client_todo));
    }

    #[test]
    fn test_state_diff_unchanged_edited() {
        let mut server_input: HashMap<Rc<PathBuf>, Rc<RefCell<File>>> = HashMap::new();
        let mut client_input: HashMap<Rc<PathBuf>, Rc<RefCell<File>>> = HashMap::new();
        let mut server_output: HashMap<Rc<PathBuf>, Rc<RefCell<File>>> = HashMap::new();
        let client_output: HashMap<Rc<PathBuf>, Rc<RefCell<File>>> = HashMap::new();

        server_input.insert(Rc::new(PathBuf::from("file")), Rc::new(RefCell::new(File::from_data(1, 2, [0; 20], State::Unchanged))));
        client_input.insert(Rc::new(PathBuf::from("file")), Rc::new(RefCell::new(File::from_data(2, 3, [1; 20], State::Edited))));
        server_output.insert(Rc::new(PathBuf::from("file")), Rc::new(RefCell::new(File::from_data(2, 3, [1; 20], State::Edited))));

        let server_data: Files = Files::from_map(server_input);
        let client_data: Files = Files::from_map(client_input);
        let server_todo: Files = Files::from_map(server_output);
        let client_todo: Files = Files::from_map(client_output);
        let result: (Files, Files) = Files::diff(server_data, client_data);

        assert_eq!(result, (server_todo, client_todo));
    }

    #[test]
    fn test_state_diff_unchanged_deleted() {
        let mut server_input: HashMap<Rc<PathBuf>, Rc<RefCell<File>>> = HashMap::new();
        let mut client_input: HashMap<Rc<PathBuf>, Rc<RefCell<File>>> = HashMap::new();
        let mut server_output: HashMap<Rc<PathBuf>, Rc<RefCell<File>>> = HashMap::new();
        let client_output: HashMap<Rc<PathBuf>, Rc<RefCell<File>>> = HashMap::new();

        server_input.insert(Rc::new(PathBuf::from("file")), Rc::new(RefCell::new(File::from_data(1, 2, [0; 20], State::Unchanged))));
        client_input.insert(Rc::new(PathBuf::from("file")), Rc::new(RefCell::new(File::from_data(2, 3, [1; 20], State::Deleted))));
        server_output.insert(Rc::new(PathBuf::from("file")), Rc::new(RefCell::new(File::from_data(2, 3, [1; 20], State::Deleted))));

        let server_data: Files = Files::from_map(server_input);
        let client_data: Files = Files::from_map(client_input);
        let server_todo: Files = Files::from_map(server_output);
        let client_todo: Files = Files::from_map(client_output);
        let result: (Files, Files) = Files::diff(server_data, client_data);

        assert_eq!(result, (server_todo, client_todo));
    }

    #[test]
    fn test_state_diff_created_unchanged() {
        let mut server_input: HashMap<Rc<PathBuf>, Rc<RefCell<File>>> = HashMap::new();
        let mut client_input: HashMap<Rc<PathBuf>, Rc<RefCell<File>>> = HashMap::new();
        let server_output: HashMap<Rc<PathBuf>, Rc<RefCell<File>>> = HashMap::new();
        let mut client_output: HashMap<Rc<PathBuf>, Rc<RefCell<File>>> = HashMap::new();

        server_input.insert(Rc::new(PathBuf::from("file")), Rc::new(RefCell::new(File::from_data(1, 2, [1; 20], State::Created))));
        client_input.insert(Rc::new(PathBuf::from("file")), Rc::new(RefCell::new(File::from_data(2, 3, [0; 20], State::Unchanged))));
        client_output.insert(Rc::new(PathBuf::from("file")), Rc::new(RefCell::new(File::from_data(1, 2, [1; 20], State::Created))));

        let server_data: Files = Files::from_map(server_input);
        let client_data: Files = Files::from_map(client_input);
        let server_todo: Files = Files::from_map(server_output);
        let client_todo: Files = Files::from_map(client_output);
        let result: (Files, Files) = Files::diff(server_data, client_data);

        assert_eq!(result, (server_todo, client_todo));
    }

    #[test]
    fn test_state_diff_created_created() {
        let mut server_input: HashMap<Rc<PathBuf>, Rc<RefCell<File>>> = HashMap::new();
        let mut client_input: HashMap<Rc<PathBuf>, Rc<RefCell<File>>> = HashMap::new();
        let server_output: HashMap<Rc<PathBuf>, Rc<RefCell<File>>> = HashMap::new();
        let client_output: HashMap<Rc<PathBuf>, Rc<RefCell<File>>> = HashMap::new();

        server_input.insert(Rc::new(PathBuf::from("file")), Rc::new(RefCell::new(File::from_data(1, 2, [0; 20], State::Created))));
        client_input.insert(Rc::new(PathBuf::from("file")), Rc::new(RefCell::new(File::from_data(2, 3, [0; 20], State::Created))));

        let server_data: Files = Files::from_map(server_input);
        let client_data: Files = Files::from_map(client_input);
        let server_todo: Files = Files::from_map(server_output);
        let client_todo: Files = Files::from_map(client_output);
        let result: (Files, Files) = Files::diff(server_data, client_data);

        assert_eq!(result, (server_todo, client_todo));
    }

    #[test]
    fn test_state_diff_created_edited() {
        let mut server_input: HashMap<Rc<PathBuf>, Rc<RefCell<File>>> = HashMap::new();
        let mut client_input: HashMap<Rc<PathBuf>, Rc<RefCell<File>>> = HashMap::new();
        let server_output: HashMap<Rc<PathBuf>, Rc<RefCell<File>>> = HashMap::new();
        let mut client_output: HashMap<Rc<PathBuf>, Rc<RefCell<File>>> = HashMap::new();

        server_input.insert(Rc::new(PathBuf::from("file")), Rc::new(RefCell::new(File::from_data(2, 3, [0; 20], State::Created))));
        client_input.insert(Rc::new(PathBuf::from("file")), Rc::new(RefCell::new(File::from_data(1, 2, [1; 20], State::Edited))));
        client_output.insert(Rc::new(PathBuf::from("file")), Rc::new(RefCell::new(File::from_data(2, 3, [0; 20], State::Created))));

        let server_data: Files = Files::from_map(server_input);
        let client_data: Files = Files::from_map(client_input);
        let server_todo: Files = Files::from_map(server_output);
        let client_todo: Files = Files::from_map(client_output);
        let result: (Files, Files) = Files::diff(server_data, client_data);

        assert_eq!(result, (server_todo, client_todo));
    }

    #[ignore = "not implemented"]
    #[test]
    fn test_state_diff_created_deleted() {
        todo!();
    }

    #[test]
    fn test_state_diff_edited_unchanged() {
        let mut server_input: HashMap<Rc<PathBuf>, Rc<RefCell<File>>> = HashMap::new();
        let mut client_input: HashMap<Rc<PathBuf>, Rc<RefCell<File>>> = HashMap::new();
        let server_output: HashMap<Rc<PathBuf>, Rc<RefCell<File>>> = HashMap::new();
        let mut client_output: HashMap<Rc<PathBuf>, Rc<RefCell<File>>> = HashMap::new();

        server_input.insert(Rc::new(PathBuf::from("file")), Rc::new(RefCell::new(File::from_data(1, 2, [1; 20], State::Edited))));
        client_input.insert(Rc::new(PathBuf::from("file")), Rc::new(RefCell::new(File::from_data(2, 2, [0; 20], State::Unchanged))));
        client_output.insert(Rc::new(PathBuf::from("file")), Rc::new(RefCell::new(File::from_data(1, 2, [1; 20], State::Edited))));

        let server_data: Files = Files::from_map(server_input);
        let client_data: Files = Files::from_map(client_input);
        let server_todo: Files = Files::from_map(server_output);
        let client_todo: Files = Files::from_map(client_output);
        let result: (Files, Files) = Files::diff(server_data, client_data);

        assert_eq!(result, (server_todo, client_todo));
    }

    #[test]
    fn test_state_diff_edited_created() {
        let mut server_input: HashMap<Rc<PathBuf>, Rc<RefCell<File>>> = HashMap::new();
        let mut client_input: HashMap<Rc<PathBuf>, Rc<RefCell<File>>> = HashMap::new();
        let mut server_output: HashMap<Rc<PathBuf>, Rc<RefCell<File>>> = HashMap::new();
        let client_output: HashMap<Rc<PathBuf>, Rc<RefCell<File>>> = HashMap::new();

        server_input.insert(Rc::new(PathBuf::from("file")), Rc::new(RefCell::new(File::from_data(1, 2, [0; 20], State::Edited))));
        client_input.insert(Rc::new(PathBuf::from("file")), Rc::new(RefCell::new(File::from_data(2, 3, [1; 20], State::Created))));
        server_output.insert(Rc::new(PathBuf::from("file")), Rc::new(RefCell::new(File::from_data(2, 3, [1; 20], State::Created))));

        let server_data: Files = Files::from_map(server_input);
        let client_data: Files = Files::from_map(client_input);
        let server_todo: Files = Files::from_map(server_output);
        let client_todo: Files = Files::from_map(client_output);
        let result: (Files, Files) = Files::diff(server_data, client_data);

        assert_eq!(result, (server_todo, client_todo));
    }

    #[test]
    fn test_state_diff_edited_edited() {
        let mut server_input: HashMap<Rc<PathBuf>, Rc<RefCell<File>>> = HashMap::new();
        let mut client_input: HashMap<Rc<PathBuf>, Rc<RefCell<File>>> = HashMap::new();
        let mut server_output: HashMap<Rc<PathBuf>, Rc<RefCell<File>>> = HashMap::new();
        let client_output: HashMap<Rc<PathBuf>, Rc<RefCell<File>>> = HashMap::new();

        server_input.insert(Rc::new(PathBuf::from("file")), Rc::new(RefCell::new(File::from_data(1, 2, [0; 20], State::Edited))));
        client_input.insert(Rc::new(PathBuf::from("file")), Rc::new(RefCell::new(File::from_data(2, 3, [1; 20], State::Edited))));
        server_output.insert(Rc::new(PathBuf::from("file")), Rc::new(RefCell::new(File::from_data(2, 3, [1; 20], State::Edited))));

        let server_data: Files = Files::from_map(server_input);
        let client_data: Files = Files::from_map(client_input);
        let server_todo: Files = Files::from_map(server_output);
        let client_todo: Files = Files::from_map(client_output);
        let result: (Files, Files) = Files::diff(server_data, client_data);

        assert_eq!(result, (server_todo, client_todo));
    }
    
    #[ignore = "not implemented"]
    #[test]
    fn test_state_diff_edited_deleted() {
        todo!();
    }

    #[ignore = "not implemented"]
    #[test]
    fn test_state_diff_deleted_unchanged() {
        todo!();
    }   

    #[ignore = "not implemented"]
    #[test]
    fn test_state_diff_deleted_created() {
        todo!();
    }

    #[ignore = "not implemented"]
    #[test]
    fn test_state_diff_deleted_edited() {
        todo!();
    }

    #[ignore = "not implemented"]
    #[test]
    fn test_state_diff_deleted_deleted() {
        todo!();
    }
}
