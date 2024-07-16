mod dep_graph;
mod extract;
mod types;

use crate::miner::dep_graph::DepGraph;
use std::path::PathBuf;

/// Build a dependency graph of the source an header files in DIRECTORY.
///
/// Currently, only *.c and *.h files are supported.
pub fn mine(directory: &PathBuf) {
    let dg = DepGraph::new(directory);
    println!("{:#?}", dg);
}
