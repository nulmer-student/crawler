use crate::compile::get_compile_bin;
use crate::pattern::{PRAGMA, LOOP_PATTERN, INFO_PATTERN};

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::io::{Error, Write};
use log::error;

/// Values returned from the loop information pass
#[derive(Debug)]
#[allow(dead_code)]
pub struct LoopInfo {
    pub line: i64,
    pub col: i64,
    pub ir_count: i64,
    pub ir_mem: i64,
    pub ir_arith: i64,
    pub ir_other: i64,
    pub pat_start: Option<i64>,
    pub pat_step: Option<i64>,
}

#[derive(Debug)]
pub struct Loop {
    row: usize,
    col: usize,
    post_row: usize, // Row after inserting the pragma
    info: Option<LoopInfo>,
}

impl Loop {
    pub fn new(row: usize, col: usize) -> Self {
        Self { row, col, post_row: row, info: None }
    }
}

#[derive(Debug)]
pub struct Loops {
    loops: Vec<Loop>,
    by_original: HashMap<usize, usize>,
}

impl Loops {
    fn from_loops(loops: Vec<Loop>) -> Self {
        let mut by_original = HashMap::new();

        // Map loops by their original row
        for (i, l) in loops.iter().enumerate() {
            by_original.insert(l.row, i);
        }

        Self { loops, by_original }
    }

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
                   Loop::new(row, col)
               })
               .collect();

        Self::from_loops(loops)
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
                    if is_for_loop(line) {
                        acc.push_str(PRAGMA);

                        // Adjust the loop lines
                        self.loops[pragma_i - 1].post_row += delta;
                        delta += 1;
                    }
                }
            }

            // Add the line
            acc.push_str(line);
            acc.push('\n');
        }

        return Ok(acc);
    }

    /// Find loop information using the "Information" pass
    pub fn loop_info(&mut self, src: &[u8], _log: &mut String) -> Result<(), ()> {
        // Spawn opt with the information pass
        let info_pass = env!("CRAWLER_SI_INFO");
        let opt = get_compile_bin("opt");
        let mut cmd = Command::new(opt)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .arg(&format!("-load-pass-plugin={}", info_pass))
            .arg("-passes=print<info>")
            .args(["-o", "/dev/null"])
            .spawn()
            .unwrap();

        // Send the optimized code to stdin
        let mut stdin = cmd.stdin.take().unwrap();
        stdin.write_all(src).unwrap();
        drop(stdin);

        // Get the output
        let output = cmd.wait_with_output().unwrap();
        let info = match String::from_utf8(output.stderr) {
            Ok(s) => s,
            Err(e) => {
                error!("Failed to read information output: {:?}", e);
                return Err(());
            },
        };

        // Parse the output into loop info structs
        let loop_info = parse_loop_info(&info);
        for li in loop_info {
            if let Some(index) = self.by_original.get(&(li.line as usize)) {
                self.loops[*index].info = Some(li);
            }
        }

        println!("{:?}", &self);
        return Ok(());
    }
}


/// Check if the given string contains a definition for a "for" loop.
fn is_for_loop(str: &str) -> bool {
    return LOOP_PATTERN.is_match(str);
}

/// Parse the loop info from INPUT.
fn parse_loop_info(input: &str) -> Vec<LoopInfo> {
    let mut acc = vec![];

    let pattern = &INFO_PATTERN;
    for (_body, [line, col, ir_count, ir_mem, ir_arith, ir_other, pat_start, pat_step])
        in pattern.captures_iter(input).map(|c| c.extract::<8>()) {

        acc.push(LoopInfo {
            line: line.parse::<i64>().unwrap(),
            col: col.parse::<i64>().unwrap(),
            ir_count: ir_count.parse::<i64>().unwrap(),
            ir_mem: ir_mem.parse::<i64>().unwrap(),
            ir_arith: ir_arith.parse::<i64>().unwrap(),
            ir_other: ir_other.parse::<i64>().unwrap(),
            pat_start: loop_info_option(pat_start),
            pat_step: loop_info_option(pat_step),
        });
    }

    return acc;
}

/// Return None if input is null, parse otherwise.
fn loop_info_option(input: &str) -> Option<i64> {
    match input {
        "null" => None,
        other => Some(other.parse::<i64>().unwrap()),
    }
}
