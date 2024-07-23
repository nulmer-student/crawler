mod compile;
mod dep_graph;
mod extract;
mod select;
mod types;

use compile::Compiler;
use dep_graph::DepGraph;
use crate::config::Config;
use crate::interface;

use std::path::PathBuf;
use rayon::{prelude::*, ThreadPool};
use log::info;

/// Build a dependency graph of the source an header files in DIRECTORY.
///
/// Currently, only `*.c` and `*.h` files are supported.
pub fn mine(directory: &PathBuf, config: &Config, pool: &ThreadPool) {
    // Build the dependency graph
    info!("Building DP graph");
    let dg = DepGraph::new(directory);

    // Load the interface
    let interface = interface::get_interface(&config.interface);

    // Compile each file
    let _ = dg.source_files().par_iter()
         .for_each(|file| {
             let compiler = Compiler::new(
                 file.clone(),
                 &dg,
                 &config,
                 interface.clone()
             );

             // Skip files for which we cannot create a compiler
             if let Ok(mut comp) = compiler {
                 comp.run();
             }
         });
}
