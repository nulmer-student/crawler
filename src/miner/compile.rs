use super::dep_graph::DepGraph;
use super::select::Selector;
use super::types::File;
use crate::config::Config;
use crate::interface::{Interface, CompileResult};

use std::path::PathBuf;
use std::sync::Arc;
use log::info;

/// This struct contains the functionality to compile a single source file.
pub struct Compiler<'a> {
    // Configuration
    config: &'a Config,
    interface: Arc<dyn Interface + Send>,

    // File we are compiling
    root_dir: &'a PathBuf,  // Directory of the repository
    file: File,             // File we are compiling
    source: String,         // String form of the file

    // Header selection
    selector: Selector<'a>,
}

impl<'a> Compiler<'a> {
    /// Create a compiler for a
    pub fn new(
        file: File,
        dg: &'a DepGraph,
        config: &'a Config,
        interface: Arc<dyn Interface + Send>
    ) -> Self {
        let root_dir = dg.root();

        // Preprocess the source file
        let source = match interface.preprocess(&root_dir, &root_dir.join(file.path())) {
            Ok(s) => s,
            Err(_) => panic!("Failed to preprocess"),
        };

        // Create the header selector
        let selector = Selector::new(file.clone(), dg, config);

        return Self { config, interface, root_dir, file, source, selector };
    }

    /// Try possible header combinations.
    pub fn run(&mut self) {
        loop {
            // Get the next possible header combination
            let Some(headers) = self.selector.step() else {
                break;
            };

            // Try to compile
            match self.try_compile(headers) {
                Ok(s) => {
                    info!("{:#?}", s);
                    break;
                },
                Err(_) => {
                    continue;
                },
            }
        }
    }

    /// Attempt to compile a single file.
    fn try_compile(&self, headers: Vec<File>) -> CompileResult {
        info!("Compile '{:?}'", self.file);

        let headers: Vec<_> = headers
            .iter()
            .map(|h| h.path().clone())
            .collect();

        return self.interface.compile(
            &self.source,
            self.root_dir,
            self.file.path(),
            &headers
        );
    }
}
