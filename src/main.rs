mod bible_lookup;

use orca::App;
use serde::Deserialize;
use serde_json::{self, Value};
use std::collections::HashMap;
use std::fs::read_to_string;
use std::io;
use toml;

fn read_config() -> io::Result<String> {
    const CONFIG_FILE: &str = "src/config.toml";
    read_to_string(CONFIG_FILE)
}


fn main() {
    let toml_str = read_config().expect("No file found");
    let config: HashMap<String, String> = toml::from_str(&toml_str).expect("Could not deserialize file");
    println!("{:?}", config);
}
