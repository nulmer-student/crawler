use std::fs;
use std::path::PathBuf;
use std::collections::HashSet;
use lazy_static::lazy_static;
use serde::Deserialize;

lazy_static! {
    static ref LANGS: HashSet<String> = {
        let mut l = HashSet::new();
        l.insert("c".to_string());
        // l.insert("cpp".to_string());
        l
    };
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub interface: String,
    pub miner: Miner,
    pub runner: Runner,
    pub database: Database,
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
    pub github_api_key: String,
}

#[derive(Debug, Deserialize)]
pub struct Database {
    pub user: String,
    pub password: String,
    pub host: String,
    pub database: String,
}

pub fn read_config(path: PathBuf) -> Config {
    let str = fs::read_to_string(path).expect("Unable to read config");

    // Load the config
    let config: Config = match toml::from_str(&str) {
        Ok(c) => c,
        Err(e) => panic!("Failed to parse config: {}", e),
    };

    // Ensure that the list of languages is valid
    let languages = &config.runner.languages;
    if !languages.is_subset(&LANGS) {
        panic!("Invalid language: {:?}", languages.difference(&LANGS))
    }

    return config;
}
