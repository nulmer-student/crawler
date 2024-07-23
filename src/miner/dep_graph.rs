use super::types::{Declare, DeclareType, File, FileType};
use super::extract::find_files;

use std::collections::{HashMap, HashSet};
use std::ffi::OsStr;
use std::path::{Component, PathBuf};
use std::fs;
use lazy_static::lazy_static;
use regex::Regex;
use log::info;

lazy_static! {
    static ref INCLUDE_PATTERN: Regex = Regex::new("#include ([\"<])([^\">]+)([\">])").unwrap();
}

type AbbrevTable = HashMap<PathBuf, Vec<File>>;

pub type Deps = HashMap<Declare, Vec<File>>;
type Edges = HashMap<File, Deps>;

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
    nodes: HashSet<File>,   // Nodes are files
    edges: Edges,           // Edges are dependencies between files
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
        let mut edges: Edges = HashMap::new();
        for file in &nodes {
            for decl in Self::parse_declare(root_dir, file) {
                // Add each possible file as an edge
                if let Some(possibilities) = abbrev.get(decl.path()) {
                    // Initialize the decl->posible table
                    if !edges.contains_key(file) {
                        edges.insert(file.clone(), HashMap::new());
                    }

                    // Insert the header file
                    if let Some(sub) = edges.get_mut(file) {
                        sub.insert(decl, possibilities.to_vec());
                    }
                };
            }
        }

        return DepGraph {
            root_dir,
            nodes,
            edges,
        };
    }

    fn parse_declare(root: &'a PathBuf, file: &File) -> Vec<Declare> {
        // Match `#include` declarations
        let pattern = &INCLUDE_PATTERN;

        // Intern each match
        let mut acc: Vec<Declare> = vec![];
        if let Ok(contents) = fs::read_to_string(root.join(file.path())) {
            for (body, [first, path, _last]) in pattern.captures_iter(&contents).map(|c| c.extract::<3>()) {
                match first {
                    "<" => {
                        acc.push(Declare::new(path, DeclareType::System))
                    },
                    "\"" => {
                        acc.push(Declare::new(path, DeclareType::User))
                    },
                    _ => {
                        panic!("Invalid header: '{}'", body);
                    }
                }
            }
        }

        return acc;
    }

    /// Return all source files in the DG
    pub fn source_files(&self) -> Vec<File> {
        let mut acc = vec![];
        for file in &self.nodes {
            if let FileType::Source = file.kind() {
                acc.push(file.clone());
            };
        }
        return acc;
    }

    // Return the root directory
    pub fn root(&self) -> &PathBuf {
        return &self.root_dir;
    }

    pub fn deps(&self, file: &File) -> Option<&Deps> {
        return self.edges.get(file);
    }
}
