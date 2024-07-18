use super::dep_graph::DepGraph;
use super::select::Selector;
use super::types::File;
use crate::config::Config;

use std::fs;
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::str;

type CompileResult = Result<String, ()>;

/// This struct contains the functionality to compile a single source file.
pub struct Compiler<'a> {
    // Configuration
    config: &'a Config,
    root_dir: &'a PathBuf,  // Directory of the repository

    // File we are compiling
    file: File,             // File we are compiling
    source: String,         // String form of the file

    // Header selection
    selector: Selector<'a>,
}

impl<'a> Compiler<'a> {
    /// Create a compiler for a
    pub fn new(file: File, dg: &'a DepGraph, config: &'a Config) -> Self {
        let root_dir = dg.root();

        // Preprocess the source file
        let source: String = match &config.miner.preprocess {
            // Run the preprocessing script if it exists
            Some(script) => {
                Self::pre_process(root_dir, &file, &script)
            },
            // Otherwise, read in the file as is
            None => {
                fs::read_to_string(&file.path()).expect("Failed to open file")
            }
        };

        // Create the header selector
        let selector = Selector::new(file.clone(), dg, config);

        return Self { config, root_dir, file, source, selector };
    }

    fn pre_process(root: &'a PathBuf, file: &File, script: &str) -> String {
        let result = Command::new(script)
            .env("ROOT", root)
            .env("FILE", root.join(file.path()))
            .output()
            .expect("Failed to run pre-processor");

        // Convert the output
        let out_str = match String::from_utf8(result.stdout) {
            Ok(s) => s,
            Err(e) => panic!("Invalid UTF-8 sequence: '{}'", e)
        };

        return out_str;
    }

    pub fn run(&mut self) {
        loop {
            // Get the next possible header combination
            let Some(headers) = self.selector.step() else {
                break;
            };

            // Try to compile
            match self.try_compile(headers) {
                Ok(_s) => { },
                Err(_) => { },
            }
        }
    }

    /// Attempt to compile a single file
    fn try_compile(&self, headers: Vec<File>) -> CompileResult {
        println!("Compile with: '{:?}'", headers);

        // // Make the headers relative to the file we are compiling
        // let header_str = "";

        // // Attempt to compile the file
        // let mut compile = Command::new(&self.config.miner.compile)
        //     .stdin(Stdio::piped())
        //     .stdout(Stdio::piped())
        //     .stderr(Stdio::piped())
        //     .env("ROOT", self.root_dir)
        //     .env("FILE", self.root_dir.join(self.file.path()))
        //     .env("HEADERS", header_str)
        //     .env("LANG", "c")
        //     .spawn()
        //     .unwrap();

        // // Send the input
        // let mut stdin = compile.stdin.unwrap();
        // let mut writer = BufWriter::new(&mut stdin);
        // writer.write_all(self.source.as_bytes()).unwrap();

        // // Get the output
        // let mut out: String = "".to_string();
        // if let Some(ref mut stdout) = compile.stdout {
        //     BufReader::new(stdout).read_to_string(&mut out).unwrap();
        // }
        // println!("{}", out);

        return Ok("".to_string());
    }
}
