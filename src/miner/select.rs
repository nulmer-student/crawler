use std::collections::{HashMap, HashSet};

use crate::config::Config;
use super::dep_graph::DepGraph;
use super::types::{Declare, File};

/// Represents an action while searching the dependency graph.
#[derive(Debug, Clone)]
enum Action {
    Start,                  // Initial node
    Foreward(File, File),   // Walk from file A to B
    Backward(File, File),   // Walk back to file A from B
}

/// Try all possible header configurations
pub struct Selector<'a> {
    // File we are selecting headers for
    file: File,

    // Graph traversal
    dg: &'a DepGraph<'a>,
    stack: Vec<Action>,             // Current path through the dep graph
    seen: HashSet<Declare>,         // Declarations that we have tried
    parents: HashMap<File, File>,   // Stores traversal parent

    // Attempts
    tries: usize,   // Number of attempts so far
    once: bool,
}

impl<'a> Selector<'a> {
    /// Create a new selector.
    pub fn new(file: File, dg: &'a DepGraph, config: &'a Config) -> Self {
        let stack = vec![Action::Start];
        let seen = HashSet::new();
        let parents = HashMap::new();

        let tries = match config.miner.tries {
            Some(n) => n,
            None => 10,
        };

        return Self { file, dg, stack, seen, parents, tries, once: false };
    }

    /// Returns the next possible header choice, or None if none are left.
    pub fn step(&mut self) -> Option<Vec<File>> {
        while self.tries > 0 {
            // Go back to the last choice
            if self.once {
                if !self.backtrack() { return None; }
            }
            self.once = true;

            // Find a new path
            while self.explore() {}

            // Get the headers
            let headers = self.get_headers();
            self.tries -= 1;
            println!("{:#?}", headers);
            return Some(headers);
        }

        // Return None if we run out of tries
        return None;
    }

    /// Explore for the next choice of headers.
    fn explore(&mut self) -> bool {
        // println!("{:#?}", self.stack);

        // Get the action on the top of the stack
        let Some(action) = self.stack.last() else {
            return false;
        };
        let action = action.clone();

        // Get the current file
        let file: File = match &action {
            Action::Start => self.file.clone(),
            Action::Foreward(_src, dest) => dest.clone(),
            Action::Backward(_src, dest) => dest.clone(),
        };

        // Find the dependencies of the current file
        let mut any_children = false;
        match self.dg.deps(&file) {
            // Explore the dependencies
            Some(deps) => {
                for (decl, possible) in deps {
                    // Don't explore children we have already seen
                    if self.seen.contains(decl) {
                        continue;
                    }

                    // Mark this child as visited
                    any_children = true;
                    self.seen.insert(decl.clone());

                    // Move to the choice
                    let child_file = possible.last().unwrap();
                    println!("{:#?}", child_file);
                    self.stack.push(
                        Action::Foreward(file.clone(), child_file.clone())
                    );

                    // Store the parent so we can backtrack
                    self.parents.insert(child_file.clone(), file.clone());
                }
            },
            None => {},
        };

        // If there are no children, try to go backward
        if !any_children {
            // Break if we get to the start node
            if let Action::Start = action {
                return false;
            }

            // Break if we try to go backward from the root
            if let Action::Backward(_src, dest) = action {
                if dest == self.file {
                    return false;
                }
            }

            // Otherwise, go backward
            let parent = self.parents.get(&file).unwrap();
            self.stack.push(Action::Backward(file, parent.clone()));
        }

        return true;
    }

    /// Backtracks, & returns true if there are more choices, & false otherwise.
    fn backtrack(&mut self) -> bool {
        return false;
    }

    /// Return the headers from a graph traversal
    fn get_headers(&self) -> Vec<File> {
        let mut acc = vec![];
        for action in &self.stack {
            match action {
                Action::Foreward(_src, dest) => {
                    acc.push(dest.clone());
                }
                _ => { },
            }
        }
        return acc;
    }
}
