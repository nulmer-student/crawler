use std::fs;
use std::path::PathBuf;
use std::collections::HashSet;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub interface: String,
    pub miner: Miner,
    pub runner: Runner,
}

#[derive(Debug, Deserialize)]
pub struct Miner {
    pub threads: usize,
    pub tries: usize,
}

#[derive(Debug, Deserialize)]
pub struct Runner {
    pub threads: usize,
    pub min_stars: usize,
    pub languages: HashSet<String>,
}

pub fn read_config(path: PathBuf) -> Config {
    let str = fs::read_to_string(path).expect("Unable to read config");

    // Load the config
    let config: Config = match toml::from_str(&str) {
        Ok(c) => c,
        Err(e) => panic!("Failed to parse config: {}", e),
    };

    // Ensure that the list of languages is valid
    let possible = HashSet::from(["c".to_string(), "cpp".to_string()]);
    let languages = &config.runner.languages;
    if !languages.is_subset(&possible) {
        panic!("Invalid language: {:?}", languages.difference(&possible))
    }

    return config;
}
