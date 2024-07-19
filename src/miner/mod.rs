mod compile;
mod dep_graph;
mod extract;
mod select;
mod types;

use compile::Compiler;
use dep_graph::DepGraph;
use crate::config::Config;
use crate::interface::{self, Interface};

use std::path::PathBuf;

/// Build a dependency graph of the source an header files in DIRECTORY.
///
/// Currently, only *.c and *.h files are supported.
pub fn mine(directory: &PathBuf, config: &Config) {
    // Build the dependency graph
    let dg = DepGraph::new(directory);

    // Load the interface
    let interface = interface::get_interface(&config.interface);

    for file in dg.source_files() {
        let mut compiler = Compiler::new(file, &dg, &config, &interface);
        compiler.run();
    }
}
