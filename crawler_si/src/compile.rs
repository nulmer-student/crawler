use crawler::interface::{CompileInput, CompileResult, MatchData};
use crate::data::Match;
use crate::loops::Loops;

use std::process::{Command, Stdio};
use std::io::Write;
use std::path::PathBuf;
use std::str::FromStr;
use log::error;

/// Get the path of a binary in the provied LLVM directory.
pub fn get_compile_bin(bin: &str) -> PathBuf {
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
    let mut loops = Loops::inner_loops(src);

    // Insert SI pragmas before the inner loops
    let pragma_src = match loops.insert_pragma(input.file) {
        Ok(s) => s,
        Err(e) => {
            error!("Failed to insert pragma: {:?}", e);
            return CompileResult { data: Err(()), to_log: log.to_string() };
        },
    };

    // Find the loop info before optimization
    match loops.loop_info(src, log) {
        Ok(_) => {},
        Err(_) => {
            error!("Failed to find loop info");
            return CompileResult { data: Err(()), to_log: log.to_string() };
        }
    }

    // Compile with SI & find remarks
    let Ok(output) = find_matches(input, pragma_src, log) else {
        return CompileResult { data: Err(()), to_log: log.to_string() };
    };

    // Parse the remarks & debug info
    loops.opt_info(&output, log);

    let result: MatchData = Box::new(Match {
        // Return the relative path
        file: input.file.strip_prefix(input.root).unwrap().to_path_buf(),
        loops,
    });

    return CompileResult { data: Ok(result), to_log: log.to_string() }
}

/// Find the SI data for a given file.
fn find_matches(input: &CompileInput, src: String, log: &mut String) -> Result<String, ()> {
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
            return Err(());
        },
    };
    log.push_str("\nOutput:\n");
    log.push_str("------------------------------\n");
    log.push_str(&output);
    log.push_str("------------------------------\n");

    // If the compilation was successful, return the stderr
    if out.status.success() {
        return Ok(output);
    }

    // If the compilation timed out, print so
    if let Some(code) = out.status.code() {
        if code == 124 {
            log.push_str("timed out\n");
        }
    }

    // Failed, this shouldn't happen since we already tried to compile
    log.push_str("failed\n");
    return Err(());
}
