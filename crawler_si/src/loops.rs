use crate::compile::get_compile_bin;
use crate::pattern::{PRAGMA, LOOP_PATTERN};

use std::fs;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::io::{Error, Write};

pub struct Loop {
    row: usize,
    col: usize,
}

pub struct Loops {
    loops: Vec<Loop>,
}

impl Loops {
    /// Find the innermost loops in SRC.
    pub fn inner_loops(src: &[u8]) -> Self {
        // Run the loop finder
        let loop_finder = env!("CRAWLER_SI_LOOPS");
        let opt = get_compile_bin("opt");
        let mut find = Command::new(opt)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .arg(&format!("-load-pass-plugin={}", loop_finder))
            .arg("-passes=print<inner-loop>")
            .args(["-o", "/dev/null"])
            .spawn()
            .unwrap();

        // Send the source file
        let mut stdin = find.stdin.take().unwrap();
        stdin.write_all(src).unwrap();
        drop(stdin);

        // Get the output
        let output = find.wait_with_output().unwrap();

        // Parse the results
        let out = String::from_utf8(output.stderr)
            .expect("Failed to parse loop finder output");

        let loops = out.lines()
               .map(|l| {
                   let pos = l.split(" ")
                              .collect::<Vec<_>>();
                   let row = pos[0].parse::<usize>().unwrap();
                   let col = pos[1].parse::<usize>().unwrap();
                   Loop {row, col}
               })
               .collect();

        Self { loops }
    }

    // Insert
    pub fn insert_pragma(&mut self, file: &PathBuf) -> Result<String, Error> {
        // Load the raw file
        let contents = fs::read_to_string(file)?;

        // Insert pragmas where needed
        let mut loop_lines: Vec<_> = self.loops.iter().map(|l| l.row).collect();
        loop_lines.sort();
        let mut acc = "".to_string();
        let mut pragma_i = 0;
        let mut delta = 1;

        for (i, line) in contents.lines().enumerate() {
            let i = i + 1; // Loop finder uses 1-based indexing

            // Check if this line needs a pragma
            if let Some(pragma_line) = loop_lines.get(pragma_i) {
                if *pragma_line == i {
                    pragma_i += 1;
                    if Self::is_for_loop(line) {
                        acc.push_str(PRAGMA);

                        // Adjust the loop lines
                        self.loops[pragma_i - 1].row += delta;
                        delta += 1;
                    }
                }
            }

            // Add the line
            acc.push_str(line);
            acc.push('\n');
        }

        println!("{}", acc);

        for l in &self.loops {
            println!("{:?}", l.row);
        }

        return Ok(acc);
    }

    fn is_inner_at_line(&self, line: usize) -> bool {
        // TODO: Make efficient
        for l in &self.loops {
            if l.row == line {
                return true;
            }
        }

        return false;
    }

    /// Check if the given string contains a definition for a "for" loop.
    fn is_for_loop(str: &str) -> bool {
        return LOOP_PATTERN.is_match(str);
    }
}
