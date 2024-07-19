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
use std::sync::Arc;
use rayon::prelude::*;

/// Build a dependency graph of the source an header files in DIRECTORY.
///
/// Currently, only *.c and *.h files are supported.
pub fn mine(directory: &PathBuf, config: &Config) {
    // Build the dependency graph
    let dg = DepGraph::new(directory);

    // Load the interface
    let interface = interface::get_interface(&config.interface);

    let files = dg.source_files().to_vec();

    let _ = files.par_iter()
         .for_each(|file| {
              let mut compiler = Compiler::new(
                  file.clone(),
                  &dg,
                  &config,
                  interface.clone()
              );
              compiler.run();
         });
}
