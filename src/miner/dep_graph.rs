use crate::miner::types::{File, Include};
use crate::miner::extract::find_files;

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

type AbbrevTable<'a> = HashMap<Include, Vec<&'a File>>;

/// Dependency graph between all source and header files in a repository.
///
/// The nodes in the graph are source and header files, and two files A & B have
/// a directed edge between them if A includes B.
#[derive(Debug)]
pub struct DepGraph<'a> {
    // Directory of the repository
    root_dir: &'a PathBuf,

    // Map include declarations to candidate files
    // FIXME: The "Include" type is wrong
    abbrev: AbbrevTable<'a>,

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
        for file in &headers {
            nodes.insert(file.clone());
        }

        // Create the abbrev table
        let abbrev = Self::build_abbrev(&headers);

        let edges = Default::default();

        return DepGraph { root_dir, abbrev, nodes, edges };
    }

    /// Build the abbrev table
    fn build_abbrev(files: &Vec<File>) -> AbbrevTable<'a> {
        let mut table = AbbrevTable::new();

        return table;
    }
}
