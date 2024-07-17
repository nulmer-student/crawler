use super::dep_graph::DepGraph;
use super::types::File;
use crate::config::Config;

use std::path::PathBuf;

pub struct Compiler<'a> {
    config: &'a Config,

    root_dir: &'a PathBuf,  // Directory of the repository
    file: File,             // File we are compiling
}

impl<'a> Compiler<'a> {
    /// Create a compiler for a
    pub fn new(file: File, dg: &'a DepGraph, config: &'a Config) -> Self {
        let root_dir = dg.root();
        return Self { config, root_dir, file };
    }

    pub fn run(&mut self) {
    }
}
