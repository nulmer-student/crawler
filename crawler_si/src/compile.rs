use crawler::interface::{CompileInput, CompileResult, MatchData};
use crate::pattern::{PRAGMA, LOOP_PATTERN};
use crate::data::Match;

use std::process::{Command, Stdio};
use std::io::{Error, Write};
use std::path::PathBuf;
use std::str::FromStr;
use std::fs;
use log::error;

/// Get the path of a binary in the provied LLVM directory.
fn get_compile_bin(bin: &str) -> PathBuf {
    let dir = PathBuf::from_str(env!("CRAWLER_SI_LLVM")).unwrap();
    return dir.join(bin);
}

/// Format headers using the -I format.
fn format_headers(headers: &Vec<PathBuf>) -> Vec<String> {
    headers.iter()
           .map(|h| format!("-I{}", h.to_str().unwrap()))
           .collect()
}

/// Return true if the compilation succeeded, & return the output.
pub fn try_compile(input: &CompileInput, log: &mut String) -> Result<Vec<u8>, ()> {
    // Get the path to clang from the args
    let clang = get_compile_bin("clang");
    let headers = format_headers(input.headers);

    // Run a quick compilation so we can check for errors
    let compile = Command::new("timeout")
        .arg("5")
        .arg(clang)
        .arg("-c")
        .arg(input.file)
        .args(headers)
        .args(["-emit-llvm", "-g", "-o", "-",])
        .output()
        .unwrap();

    // Log the command used
    log.push_str("\n==============================\n");
    log.push_str(
        &format!("Try for file {:?}:\nHeaders: {:?}", input.file, input.headers)
    );

    // Try to get the output
    let output = match String::from_utf8(compile.stderr) {
        Ok(s) => s,
        Err(e) => {
            error!("Failed to read compilation output for: {:?}", e);
            "".to_string()
        }
    };
    log.push_str("\nOutput:\n");
    log.push_str("------------------------------\n");
    log.push_str(&output);
    log.push_str("------------------------------\n");

    // If the compilation timed out, print so
    if let Some(code) = compile.status.code() {
        if code == 124 {
            log.push_str("timed out\n");
        }
    }

    // Return true if the compilation succeeded
    let result = match compile.status.success() {
        true => Ok(compile.stdout),
        false => Err(()),
    };

    if result.is_ok() {
        log.push_str("success\n")
    } else {
        log.push_str("failed\n")
    }

    return result;
}

/// Given a successful header combination, compile the file & find matches.
pub fn find_match_data(input: &CompileInput, log: &mut String, src: &[u8]) -> CompileResult {
    // Find the innermost loops in the file
    let loop_lines = find_inner_loops(src);

    // Insert SI pragmas before the inner loops
    let src = match insert_pragma(input.file, loop_lines) {
        Ok(s) => s,
        Err(e) => {
            error!("Failed to insert pragma: {:?}", e);
            return CompileResult { data: Err(()), to_log: log.to_string() };
        },
    };

    // Compile to find all information
    return find_matches(input, src, log);
}

/// Return a list of line numbers that define innermost loops.
fn find_inner_loops(src: &[u8]) -> Vec<usize> {
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
    let lines: Vec<_> = out.lines()
                           .map(|l| l.split(" ").collect::<Vec<_>>()[0])
                           .filter_map(|s| s.parse::<usize>().ok())
                           .collect();
    return lines;
}

/// Load a file into a String & insert the SI pragmas.
fn insert_pragma(file: &PathBuf, mut pragma_lines: Vec<usize>) -> Result<String, Error> {
    // Load the raw file
    let contents = fs::read_to_string(file)?;

    // Ensure the pragma_lines are in sorted order
    pragma_lines.sort();

    // Load the file & insert the pragmas where needed
    let mut acc = "".to_string();
    let mut pragma_i = 0;
    for (i, line) in contents.lines().enumerate() {
        let i = i + 1; // Loop finder uses 1-based indexing

        // Check if this line needs a pragma
        if let Some(pragma_line) = pragma_lines.get(pragma_i) {
            if *pragma_line == i {
                pragma_i += 1;
                if is_for_loop(line) {
                    acc.push_str(PRAGMA);
                }
            }
        }

        // Add the line
        acc.push_str(line);
        acc.push('\n');
    }

    return Ok(acc);
}

/// Check if the given string contains a definition for a "for" loop.
fn is_for_loop(str: &str) -> bool {
    return LOOP_PATTERN.is_match(str);
}

/// Find the SI data for a given file.
fn find_matches(input: &CompileInput, src: String, log: &mut String) -> CompileResult {
    let mut compile = Command::new("timeout")
        .arg("10")
        .arg(get_compile_bin("clang"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .args(["-c", "-x", "c"])
        .args(format_headers(input.headers))
        .args(["-o", "-"])
        .args(["-emit-llvm", "-O3", "-Rpass=loop-vectorize"])
        .args(["-mllvm", "-debug-only=loop-vectorize"])
        .arg("-")
        .spawn()
        .unwrap();

    // Log the command used
    log.push_str("\n==============================\n");
    log.push_str(
        &format!("Finding info for file {:?}:\nHeaders: {:?}",
                 input.file, input.headers)
    );

    // Send the source file
    let mut stdin = compile.stdin.take().unwrap();
    stdin.write_all(src.as_bytes()).unwrap();
    drop(stdin);

    // Get the compilation output
    let out = compile.wait_with_output().unwrap();

    // Get the output as a string
    let output = match String::from_utf8(out.stderr) {
        Ok(s) => s,
        Err(e) => {
            error!("Failed to read match data: {}", e);
            return CompileResult { data: Err(()), to_log: log.to_string() };
        },
    };
    log.push_str("\nOutput:\n");
    log.push_str("------------------------------\n");
    log.push_str(&output);
    log.push_str("------------------------------\n");

    // If the compilation was successful, return the stderr
    if out.status.success() {
        // Run the loop info pass
        let info = match loop_info(&out.stdout, log) {
            Ok(s) => s,
            Err(e) => {
                error!("Failed to find loop info: {:?}", e);
                "".to_string()
            }
        };
        log.push_str(&info);
        log.push_str("------------------------------\n");

        // Return the result
        let result: MatchData = Box::new(Match {
            // Return the relative path
            file: input.file.strip_prefix(input.root).unwrap().to_path_buf(),
            output: output + &info,
        });
        log.push_str("success\n");
        return CompileResult { data: Ok(result), to_log: log.to_string() };
    }

    // If the compilation timed out, print so
    if let Some(code) = out.status.code() {
        if code == 124 {
            log.push_str("timed out\n");
        }
    }

    // Failed, this shouldn't happen since we already tried to compile
    log.push_str("failed\n");
    return CompileResult { data: Err(()), to_log: log.to_string() };
}

/// Find loop information using the "Information" pass
fn loop_info(src: &[u8], _log: &mut String) -> Result<String, ()> {
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

    return Ok(info);
}
