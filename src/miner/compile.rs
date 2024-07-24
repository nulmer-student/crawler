use super::dep_graph::DepGraph;
use super::select::Selector;
use super::types::File;
use crate::config::Config;
use crate::interface::{CompileInput, CompileResult, Interface, InternInput, MatchData, PreInput};

use std::path::PathBuf;
use std::sync::Arc;
use log::{error, debug};

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

        return Ok(Self {
            config, interface, root_dir, file, source, selector
        });
    }

    /// Try possible header combinations.
    pub fn run(&mut self) {
        let mut match_data: Option<MatchData> = None;

        loop {
            // Get the next possible header combination
            let Some(headers) = self.selector.step() else {
                break;
            };

            // Try to compile
            match self.try_compile(headers) {
                Ok(s) => {
                    match_data = Some(s);
                    break;
                },
                Err(_) => {
                    debug!("Failed to compile '{:?}'", self.file);
                    continue;
                },
            }
        }

        // If there are any matches, intern them
        if let Some(data) = match_data {
            debug!("Interning results");
            let input = InternInput {
                config: self.config,
                root: self.root_dir,
                file: &self.file_full(),
                data: &data,
            };
            match self.interface.intern(input) {
                Ok(_) => {},
                Err(e) => error!("Failed to intern: {:?}", e),
            }
        }
    }

    /// Attempt to compile a single file.
    fn try_compile(&self, headers: Vec<File>) -> CompileResult {
        debug!("Compile '{:?}'", self.file);

        let input = CompileInput {
            config: self.config,
            root: self.root_dir,
            file: &self.file_full(),
            content: &self.source,
            headers: &self.qualify_headers(headers),
        };
        return self.interface.compile(input);
    }

    /// Make headers relative to the current file.
    fn qualify_headers(&self, headers: Vec<File>) -> Vec<PathBuf> {
        let headers: Vec<_> = headers
            .iter()
            .map(|h| h.path().clone())
            .collect();
        return headers;
    }

    /// Return the full path of the current file.
    fn file_full(&self) -> PathBuf {
        return self.root_dir.join(self.file.path());
    }
}
