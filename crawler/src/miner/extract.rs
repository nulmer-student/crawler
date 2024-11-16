use crate::miner::types::File;

use std::path::PathBuf;
use std::process::Command;
use std::str;

/// Find all files matching PATTERN in a given DIRECTORY.
pub fn find_files(directory: &PathBuf, pattern: &str) -> Vec<File> {
    // Run the `find` command
    let out = Command::new("find")
        .arg(directory)
        .arg("-name")
        .arg(pattern)
        .output()
        .expect("Failed to find files");

    // Covert to a utf-8 string
    let out_str = match str::from_utf8(&out.stdout) {
        Ok(s) => s,
        Err(e) => panic!("Invalid UTF-8 sequence: '{}'", e),

    };

    // Split lines & convert to file objects
    let mut acc: Vec<File> = vec![];
    for line in out_str.lines() {
        // Skip files that fail
        if let Some(f) = File::relative(line, directory) {
            acc.push(f);
        }
    }

    return acc;
}
