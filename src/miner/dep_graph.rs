use crate::miner::types::{File, FileType};
use crate::miner::extract::find_files;

use std::collections::{HashMap, HashSet};
use std::ffi::OsStr;
use std::path::{Component, PathBuf};
use std::fs;

use super::types::Declare;

type AbbrevTable = HashMap<PathBuf, Vec<File>>;

/// Dependency graph between all source and header files in a repository.
///
/// The nodes in the graph are source and header files, and two files A & B have
/// a directed edge between them if A includes B.
#[derive(Debug)]
pub struct DepGraph<'a> {
    // Directory of the repository
    root_dir: &'a PathBuf,

    // Map possible include paths to candidate files
    // abbrev: AbbrevTable,

    // Graph structure
    nodes: HashSet<File>,                   // Nodes are files
    edges: HashMap<File, HashSet<File>>     // Edges are dependencies between files
}

impl<'a> DepGraph<'a> {
    /// Create a new dependency graph rooted at ROOT_DIR.
    pub fn new(root_dir: &'a PathBuf) -> Self {
        // Find the source files in the repository
        let src     = find_files(root_dir, "*.c");
        let headers = find_files(root_dir, "*.h");

        // Insert all files as nodes
        let mut nodes = HashSet::new();
        for file in src {
            nodes.insert(file);
        }
        for file in headers {
            nodes.insert(file);
        }

        // For each file, add the the possible declarations to the table
        let mut abbrev: AbbrevTable = Default::default();
        for file in &nodes {
            // Only consider source files
            match file.kind() {
                FileType::Source => { continue; },
                _ => {},
            };

            let mut acc: Vec<&OsStr> = vec![];

            // Reverse the components & accumulate, for example: "a/b/c.h"
            // interns the following paths:
            // - "c.h"
            // - "b/c.h"
            // - "a/b/c.h"
            for c in file.components().rev() {
                if let Component::Normal(comp) = c {
                    acc.insert(0, comp);        // FIXME: O(n) insert
                    let path: PathBuf = acc.iter().collect();
                    match abbrev.get_mut(&path) {
                        Some(v) => {
                            v.push(file.clone());
                        },
                        None => {
                            abbrev.insert(path, vec![file.clone()]);
                        },
                    };
                }
            }
        }

        // For each file, add edges where there are dependencies
        for file in &nodes {
            let decl = Self::parse_declare(file);
            println!("{:#?}", decl);
        }

        return DepGraph {
            root_dir,
            nodes,
            edges: Default::default(),
        };
    }

    fn parse_declare(file: &File) -> Vec<Declare> {
        let mut acc: Vec<Declare> = vec![];

        let contents = fs::read_to_string(file.path());
        println!("{:#?}", contents);

        return acc;
    }
}
