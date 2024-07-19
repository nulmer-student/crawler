use std::{fs, path::PathBuf};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub interface: String,
    pub miner: Miner,
}

#[derive(Debug, Deserialize)]
pub struct Miner {
    pub threads: usize,
    pub tries: usize,
}

#[derive(Debug, Deserialize)]
pub struct Runner {
    pub threads: usize,
}

pub fn read_config(path: PathBuf) -> Config {
    let str = fs::read_to_string(path).expect("Unable to read config");
    let config: Config = match toml::from_str(&str) {
        Ok(c) => c,
        Err(e) => panic!("Failed to parse config: {}", e),
    };
    return config;
}
