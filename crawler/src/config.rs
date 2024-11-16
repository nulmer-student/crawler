use std::fs;
use std::path::PathBuf;
use std::collections::HashSet;
use lazy_static::lazy_static;
use serde::Deserialize;
use chrono::Local;

lazy_static! {
    static ref LANGS: HashSet<String> = {
        let mut l = HashSet::new();
        l.insert("c".to_string());
        // l.insert("cpp".to_string());
        l
    };
}

/// Top level configuration.
#[derive(Clone, Debug, Deserialize)]
pub struct Config {
    pub miner: Miner,
    pub runner: Runner,
    pub database: Database,
}

/// Configuration for the miner.
#[derive(Clone, Debug, Deserialize)]
pub struct Miner {
    pub threads: usize,
    pub tries: usize,
}

/// Configuration for the runner.
#[derive(Clone, Debug, Deserialize)]
pub struct Runner {
    pub threads: usize,
    pub min_stars: usize,
    pub languages: HashSet<String>,
    pub github_api_key: String,
    pub log_dir: PathBuf,
    pub log_level: String,
    pub tmp_dir: PathBuf,
}

/// Configuration for the database.
#[derive(Clone, Debug, Deserialize)]
pub struct Database {
    pub user: String,
    pub password: String,
    pub host: String,
    pub database: String,
}

pub fn read_config(path: PathBuf) -> Config {
    let str = fs::read_to_string(path).expect("Unable to read config");

    // Load the config
    let mut config: Config = match toml::from_str(&str) {
        Ok(c) => c,
        Err(e) => panic!("Failed to parse config: {}", e),
    };

    // Ensure that the list of languages is valid
    let languages = &config.runner.languages;
    if !languages.is_subset(&LANGS) {
        panic!("Invalid language: {:?}", languages.difference(&LANGS))
    }

    // Set the log directory based on the time
    let now = Local::now();
    let sub_dir = format!("{:?}", now);
    config.runner.log_dir = config.runner.log_dir.join(sub_dir);

    return config;
}
