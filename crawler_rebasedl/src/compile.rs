use log::error;

use std::process::{Command, Stdio};
use std::io::{Error, Write};
use std::str::FromStr;
use std::path::PathBuf;

use crawler::interface::{CompileInput, CompileResult};

use crate::output_parser;

/// Return a compilation error.
fn compile_fail(log: &mut String) -> CompileResult {
    CompileResult { data: Err(()), to_log: log.to_string() }
}

/// Get the path of a binary in the provied LLVM directory.
fn get_compile_bin(bin: &str) -> PathBuf {
    let dir = PathBuf::from_str("/home/nju/.opt/llvm-17/llvm-bin/bin").unwrap();
    return dir.join(bin);
}

/// Format headers using the -I format.
fn format_headers(headers: &Vec<PathBuf>) -> Vec<String> {
    headers.iter()
           .map(|h| format!("-I{}", h.to_str().unwrap()))
           .collect()
}

fn compile_file(input: &CompileInput, log: &mut String) -> Result<Vec<u8>, ()> {
    // Get the path to clang from the args
    let clang = get_compile_bin("clang");
    let headers = format_headers(input.headers);

    // Run a quick compilation so we can check for errors
    let compile = Command::new("timeout")
        .arg("10")
        .arg(clang)
        .args(["-g", "-O3", "-fno-unroll-loops", "-emit-llvm", "-S"])
        .args(["-o", "-"])
        .arg(input.file)
        .args(headers)
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

/// Run the RebaseDL pass on input LLVM IR.
fn run_rebasedl_pass(src: &[u8]) -> Result<String, ()> {
    let opt = get_compile_bin("opt");
    let pass = "/home/nju/.opt/rebasedl-pass/build/lib/libRebaseDLPass.so";

    let mut cmd = Command::new(opt)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .arg("-disable-output")
        .arg(&format!("-load-pass-plugin={}", pass))
        .arg("-passes=rebasedl")
        .spawn()
        .unwrap();

    // Send the source file
    let mut stdin = cmd.stdin.take().unwrap();
    stdin.write_all(src).unwrap();
    drop(stdin);

    // Get the output
    let output = cmd.wait_with_output().unwrap();
    let out = String::from_utf8(output.stderr)
        .expect("Failed to parse pass output");

    Ok(out)
}

/// Try to compile a file, & return the match data if successful.
pub fn try_compile(input: &CompileInput, log: &mut String) -> CompileResult {
    // Compile the file
    let src = match compile_file(input, log) {
        Ok(src) => src,
        Err(_) => return compile_fail(log),
    };

    // Run the RebaseDL pass
    let pass_output = match run_rebasedl_pass(&src) {
        Ok(out) => out,
        Err(_) => return compile_fail(log),
    };

    // Parse the results
    let data = output_parser::parse(pass_output);
    CompileResult { data: Ok(Box::new(data)), to_log: log.to_string() }
}
