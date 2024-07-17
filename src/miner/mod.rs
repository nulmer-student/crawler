mod compile;
mod dep_graph;
mod extract;
mod types;

use compile::Compiler;
use dep_graph::DepGraph;
use crate::config::Config;

use std::path::PathBuf;

/// Build a dependency graph of the source an header files in DIRECTORY.
///
/// Currently, only *.c and *.h files are supported.
pub fn mine(directory: &PathBuf, config: &Config) {
    let dg = DepGraph::new(directory);

    for file in dg.source_files() {
        let mut compiler = Compiler::new(file, &dg, &config);
        compiler.run();
    }
}
