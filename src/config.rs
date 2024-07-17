use std::{fs, path::PathBuf};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub miner: Miner,
}

#[derive(Debug, Deserialize)]
pub struct Miner {
    pub compile: String,
    pub preprocess: Option<String>,
}

pub fn read_config(path: PathBuf) -> Config {
    let str = fs::read_to_string(path).expect("Unable to read config");
    let config: Config = toml::from_str(&str).unwrap();
    return config;
}
