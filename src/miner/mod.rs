mod compile;
mod dep_graph;
mod extract;
mod select;
mod types;

use compile::Compiler;
use dep_graph::DepGraph;
use crate::config::Config;
use crate::interface::{self, MatchData};

use std::path::PathBuf;
use rayon::prelude::*;

/// Build a dependency graph of the source an header files in DIRECTORY.
///
/// Currently, only `*.c` and `*.h` files are supported.
pub fn mine(directory: &PathBuf, config: Config) {
    // Build the dependency graph
    let dg = DepGraph::new(directory);

    // Load the interface
    let interface = interface::get_interface(&config.interface.name);

    // Compile each file
    let match_data: Vec<MatchData> = dg.source_files().par_iter()
        .filter_map(|file| {
            // Try to compile the file
            let mut compiler = Compiler::new(
                 file.clone(),
                 &dg,
                 &config,
                 interface.clone()
             );
            return compiler.run();
         }).collect();

    // Intern the matches
    // TODO
}

/// Mine a single repository, using a dedicated thread pool.
pub fn mine_one(directory: PathBuf, config: Config) {
    rayon::ThreadPoolBuilder::new()
        .num_threads(config.miner.threads)
        .thread_name(|i| format!("mine-{}", i))
        .build_global()
        .expect("Failed to create miner thread pool");

    mine(&directory, config);
}
