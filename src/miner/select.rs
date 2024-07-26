use std::collections::{HashMap, HashSet};

use log::{trace, error};

use crate::config::Config;
use super::dep_graph::DepGraph;
use super::types::{Declare, File};

/// Represents an action while searching the dependency graph.
#[derive(Debug, Clone)]
enum Action {
    Start,                  // Initial node
    Foreward(File, File),   // A -> B
    Backward(File, File),   // B -> A
    Many(File, Vec<File>),  // A -> {B, C, D, ...}
}

/// Try all possible header configurations
pub struct Selector<'a> {
    // File we are selecting headers for
    file: File,

    // Graph traversal
    dg: &'a DepGraph<'a>,
    stack: Vec<Action>,             // Current path through the dep graph
    seen: HashMap<File, Declare>,   // Declarations that we have tried
    parents: HashMap<File, File>,   // Stores tree parent

    // Attempts
    tries: usize,   // Number of attempts so far
    once: bool,     // True if we have tried at least once
}

impl<'a> Selector<'a> {
    /// Create a new selector.
    pub fn new(file: File, dg: &'a DepGraph, config: &'a Config) -> Self {
        let stack = vec![Action::Start];
        let seen    = Default::default();
        let parents = Default::default();

        let tries = config.miner.tries;

        return Self { file, dg, stack, seen, parents, tries, once: false };
    }

    /// Returns the next possible header choice, or None if none are left.
    pub fn step(&mut self) -> Option<Vec<(File, Declare)>> {
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
            return Some(headers);
        }

        // Return None if we run out of tries
        return None;
    }

    /// Explore for the next choice of headers.
    fn explore(&mut self) -> bool {
        trace!("===== Explore =====");
        trace!("Stack: {:#?}", self.stack);
        trace!("Parents: {:#?}", self.parents);
        trace!("Seen: {:#?}", self.seen);

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
            Action::Many(_src, possible) => {
                possible.last().unwrap().clone()
            },
        };

        // Find the dependencies of the current file
        let mut any_children = false;
        match self.dg.deps(&file) {
            // Explore the dependencies
            Some(deps) => {
                for (decl, possible) in deps {
                    let found = self.visit(&file, &decl, possible);

                    // Don't explore any other children
                    if found {
                        any_children = true;
                        break;
                    }
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

    fn visit(&mut self, file: &File, decl: &Declare, possible: &[File]) -> bool {
        // Don't visit if we have seen any of the possibilities
        for p in possible {
            if self.seen.contains_key(p) {
                return false;
            }
        }

        // Mark this child as visited
        let child_file = possible.last().unwrap(); // Always at least one
        self.seen.insert(child_file.clone(), decl.clone());

        // Store the parent so we can backtrack
        self.parents.insert(child_file.clone(), file.clone());

        // Move to the choice
        match possible.len() {
            0 => { unreachable!() },
            // If there is only one possibility, insert a Foreward
            1 => {
                self.stack.push(
                    Action::Foreward(file.clone(), child_file.clone())
                );

            },
            // Otherwise, insert a Many
            _ => {
                self.stack.push(
                    Action::Many(file.clone(), possible.to_vec())
                );
            }
        }

        return true;
    }

    /// Backtracks, & returns true if there are more choices, & false otherwise.
    fn backtrack(&mut self) -> bool {
        loop {
            trace!("===== Backtrack =====");
            trace!("Stack: {:#?}", self.stack);
            trace!("Parents: {:#?}", self.parents);
            trace!("Seen: {:#?}", self.seen);

            // Get the action on the top of the stack
            let Some(action) = self.stack.last() else {
                return false;
            };
            let action = action.clone();


            match action {
                // If we backtrack to the start, there are no more choices
                Action::Start => {
                    return false;
                },

                // If we are at a choice point, go to the next choice
                Action::Many(src, possible) => {
                    if let Some((last, rest)) = possible.split_last() {
                        // Remove the current choice
                        let decl = self.seen.get(&last).unwrap().clone();
                        self.seen.remove(&last);

                        self.stack.pop();
                        if rest.len() > 0 {
                            // Remove the last possibility
                            self.stack.push(Action::Many(
                                src.clone(), rest.to_vec())
                            );

                            let new = rest.last().unwrap();

                            // Fix the parent
                            self.parents.insert(
                                new.clone(),
                                src.clone()
                            );

                            // Add to the seen table
                            self.seen.insert(new.clone(), decl.clone());

                            // Return that there are more possibilities
                            return true;
                        }
                    }
                },

                // Otherwise, remove the element
                Action::Foreward(_src, dest) => {
                    self.seen.remove(&dest);
                    self.stack.pop();
                },
                Action::Backward(_src, _dest) => {
                    self.stack.pop();
                },
            }
        }
    }

    /// Return the headers from a graph traversal
    fn get_headers(&self) -> Vec<(File, Declare)> {
        let mut acc = vec![];

        for action in &self.stack {
            // Collect the files we visit from forewards
            let (dest, decl) = match action {
                Action::Foreward(_src, dest) => {
                    (dest, self.seen.get(&dest))
                }
                Action::Many(_src, possible) => {
                    let dest = possible.last().unwrap();
                    (dest, self.seen.get(&dest))
                }
                _ => { continue; },
            };

            // Add the file
            if let Some(in_decl) = decl {
                acc.push((dest.clone(), in_decl.clone()));
            } else {
                error!("Seen table is missing declaration for: {:?}", dest);
            }
        }

        return acc;
    }
}
