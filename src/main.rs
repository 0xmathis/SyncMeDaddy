use std::path::{Path, PathBuf};
use std::collections::HashMap;

use utils::{get_current_state, time_since_epoch};
use utils::tree_directory;
use utils::my_json::{
    File, JSON
};


fn main() {
    let datapath: PathBuf = Path::new("/home/mathis/SyncMeDaddy").to_path_buf();
    // tree_directory(datapath);
    
    get_current_state(datapath);

    // let datapath: PathBuf = Path::new("/home/mathis/test.json").to_path_buf();
    // let json: JSON = JSON::load_from_file(datapath).unwrap();
    // let output: &HashMap<String, File> = json.get_data();
    // println!("{:?}", output);
    // println!("{:?}", output.keys());

    // for (key, value) in output {
    //     println!("{} : {:?}", key, value);
    // }

    // println!("Now : {}", time_since_epoch());
}
