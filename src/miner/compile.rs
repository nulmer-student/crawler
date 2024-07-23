use super::dep_graph::DepGraph;
use super::select::Selector;
use super::types::File;
use crate::config::Config;
use crate::interface::{CompileInput, CompileResult, Interface, InternInput, MatchData, PreInput};

use std::path::PathBuf;
use std::sync::Arc;
use log::{info, error, debug};

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

    // Match data
    matches: Vec<MatchData>,
}

impl<'a> Compiler<'a> {
    /// Create a compiler for a
    pub fn new(
        file: File,
        dg: &'a DepGraph,
        config: &'a Config,
        interface: Arc<dyn Interface + Send>
    ) -> Result<Compiler<'a>, ()> {
        let root_dir = dg.root();

        let input = PreInput {
            config,
            root: &root_dir,
            file: &root_dir.join(file.path()),
        };

        // Preprocess the source file
        let source = match interface.preprocess(input) {
            Ok(s) => s,
            Err(_) => {
                error!("Failed to preprocess '{:?}'", file.path());
                return Err(())
            },
        };

        // Create the header selector
        let selector = Selector::new(file.clone(), dg, config);
        let matches = vec![];

        return Ok(Self {
            config, interface, root_dir, file, source, selector, matches
        });
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
                Ok(mut s) => {
                    self.matches.append(&mut s);
                    break;
                },
                Err(_) => {
                    continue;
                },
            }
        }

        // If there are any matches, intern them
        debug!("Interning results ({})", self.matches.len());
        let input = InternInput {
            config: self.config,
            root: self.root_dir,
            file: self.file.path(),
            data: &self.matches,
        };
        match self.interface.intern(input) {
            Ok(_) => {},
            Err(e) => error!("Failed to intern: {:?}", e),
        }
    }

    /// Attempt to compile a single file.
    fn try_compile(&self, headers: Vec<File>) -> CompileResult {
        debug!("Compile '{:?}'", self.file);

        let headers: Vec<_> = headers
            .iter()
            .map(|h| h.path().clone())
            .collect();

        let input = CompileInput {
            config: self.config,
            root: self.root_dir,
            file: self.file.path(),
            content: &self.source,
            headers: &headers
        };
        return self.interface.compile(input);
    }
}
