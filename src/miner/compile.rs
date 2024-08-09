use super::dep_graph::DepGraph;
use super::select::Selector;
use super::types::{Declare, File};
use crate::config::Config;
use crate::interface::{
    CompileInput, CompileResult, Interface, MatchData, PreInput
};

use std::collections::HashSet;
use std::panic::{self, AssertUnwindSafe};
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

    // Header selection
    selector: Selector<'a>,
    tried: HashSet<Vec<PathBuf>>,

    // Log data
    log_data: String,
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

        // Create the header selector
        let selector = Selector::new(file.clone(), dg, config);
        let tried = HashSet::new();

        // Start out with an empty log
        let log_data = "".to_string();

        return Self {
            config, interface, root_dir, file, selector, tried, log_data
        };
    }

    /// Try possible header combinations.
    pub fn run(&mut self) -> Result<MatchData, String> {
        // Preprocess the source file
        let input = PreInput {
            config: self.config,
            root: &self.root_dir,
            file: &self.file_full(),
        };

        let source = match self.interface.preprocess(input) {
            Ok(s) => s,
            Err(_) => {
                error!("Failed to preprocess {:?}", self.file.path());
                return Err("Failed to preprocess".to_string());
            },
        };

        // Compile the file
        loop {
            // Get the next possible header combination
            let Some(headers) = self.selector.step() else {
                return Err("Ran out of header possibilities".to_string());
            };

            // Don't try any header combination more than once
            let headers = self.qualify_headers(headers);
            debug!("Headers: {:?}", headers);
            if self.tried.contains(&headers) {
                debug!("Already seen headers: {:?}", headers);
                continue;
            }
            self.tried.insert(headers.clone());

            // Try to compile
            let result = self.try_compile(&source, headers);
            self.log(&result.to_log);

            // Exit if we have succeeded, keep trying otherwise
            match result.data {
                Ok(s) => {
                    return Ok(s);
                },
                Err(_) => {
                    debug!("Failed to compile {:?}", self.file.path());
                    continue;
                },
            }
        }
    }

    /// Attempt to compile a single file.
    fn try_compile(&self, source: &str, headers: Vec<PathBuf>) -> CompileResult {
        debug!("Compile {:?}", self.file.path());

        let input = CompileInput {
            config: self.config,
            root: self.root_dir,
            file: &self.file_full(),
            content: source,
            headers: &headers,
        };

        match panic::catch_unwind(AssertUnwindSafe(|| { self.interface.compile(input) })) {
            Ok(r) => r,
            Err(_) => {
                error!("Panic during try_compile");
                CompileResult { data: Err(()), to_log: "".to_string() }
            }
        }
    }

    /// Make headers relative to the current file.
    fn qualify_headers(&self, headers: Vec<(File, Declare)>) -> Vec<PathBuf> {
        // FIXME: Remove system headers
        let headers: Vec<_> = headers
            .iter()
            .filter(|h| h.1.is_user())
            .collect();

        // Get the absolute paths of the headers
        let abs: Vec<_> = headers
            .iter()
            .map(|h| self.root_dir.join(h.0.path()))
            .collect();

        // Remove the declaration part of each header
        let mut acc: Vec<PathBuf> = vec![];
        for i in 0..headers.len() {
            // Find the length of the declaration
            let decl_parts: Vec<_> = headers[i].1.path().components().collect();
            let size = decl_parts.len();

            // Remove from the end SIZE components.
            let header_parts: Vec<_> = abs[i].components().rev().collect();
            let rest = &header_parts[size..];

            // Rebuild the path
            let path = PathBuf::from_iter(rest.iter().rev());
            acc.push(path)
        }

        // Remove any duplicates
        let mut minimal = vec![];
        let mut seen: HashSet<PathBuf> = HashSet::new();

        for h in acc {
            if seen.contains(&h) { continue; }
            seen.insert(h.clone());
            minimal.push(h);
        }

        // Sort, to allow for easy easy duplicate checking
        minimal.sort_unstable();
        return minimal;
    }

    /// Return the full path of the current file.
    fn file_full(&self) -> PathBuf {
        return self.root_dir.join(self.file.path());
    }

    /// Add the input string to the log.
    fn log(&mut self, input: &str) {
        self.log_data.push_str(input);
    }

    pub fn get_log(&self) -> &str {
        return &self.log_data;
    }
}
