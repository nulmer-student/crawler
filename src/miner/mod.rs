mod compile;
mod dep_graph;
mod extract;
mod select;
mod types;

use compile::Compiler;
use dep_graph::DepGraph;
use crate::config::Config;
use crate::interface::{self, MatchData};

use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, mpsc};
use rayon::prelude::*;
use log::{debug, info, error};

pub struct MineResult {
    pub data: Vec<MatchData>,
    pub n_files: i64,
    pub n_success: i64,
    pub n_error: i64,
}

/// Build a dependency graph of the source an header files in DIRECTORY.
///
/// Currently, only `*.c` and `*.h` files are supported.
pub fn mine(directory: &PathBuf, log_file: &PathBuf, config: Config) -> Result<MineResult, ()> {
    // Build the dependency graph
    let dg = DepGraph::new(directory);

    // Load the interface
    let interface = interface::get_interface(&config.interface.name);

    // Open the log file
    let mut log = match fs::OpenOptions::new()
        .append(true)
        .create(true)
        .open(log_file) {
        Ok(f) => f,
        Err(e) => {
            error!("Failed to open log file: {:?}", e);
            return Err(());
        },
    };
    let log = Arc::new(Mutex::new(log));

    // Count the number of files that fail
    let total: i64 = dg.source_files().len() as i64;
    let (tx, rx) = mpsc::channel::<i64>();

    // Compile each file
    info!("Starting compilation");
    let match_data: Vec<MatchData> = dg.source_files().par_iter()
        .filter_map(|file| {
            let tx = tx.clone();

            // Try to compile the file
            let mut compiler = Compiler::new(
                 file.clone(),
                 &dg,
                 &config,
                 interface.clone()
             );
            let result = match compiler.run() {
                Ok(data) => {
                    tx.send(1).unwrap();
                    Some(data)
                },
                Err(e) => {
                    debug!("Failed completely for {:?}: {}", file.path(), e);
                    tx.send(0).unwrap();
                    None
                },
            };

            // Send the compiler output
            if let Some(out) = &result {
                let mut outfile = log.lock().unwrap();
                outfile.write_all(out.to_log.as_bytes()).unwrap();
            }

            drop(tx);
            return result;
         }).map(|r| r.data)
           .collect();

    // Gather the counts
    drop(tx);
    let success: i64 = rx.iter().sum();
    info!("Results: total: {}, successful: {}", total, success);

    return Ok(MineResult {
        data: match_data,
        n_files: total,
        n_success: success,
        n_error: total - success,
    });
}

/// Mine a single repository, using a dedicated thread pool.
pub fn mine_one(directory: PathBuf, config: Config) {
    rayon::ThreadPoolBuilder::new()
        .num_threads(config.miner.threads)
        .thread_name(|i| format!("mine-{}", i))
        .build_global()
        .expect("Failed to create miner thread pool");

    let log_file = config.runner.log_dir.join("repo.log");
    let _ = mine(&directory, &log_file, config);
}
