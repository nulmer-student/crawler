use std::process::Command;
use std::str::FromStr;
use std::path::PathBuf;

use log::error;
use lazy_static::lazy_static;
use regex::Regex;

use crawler::interface::{CompileInput, CompileResult};
use crate::data::{KernelMatch, Match};

const BIN: &str = "/home/nju/.opt/KernelFaRer/build/install/bin";

lazy_static! {
    pub static ref MATCH: Regex = {
        let mut pattern = "".to_string();
        pattern.push_str(r"Kernel rewritable at line (\d+) with type (\d+)");
        Regex::new(&pattern).unwrap()
    };
}

/// Return a compilation error.
fn compile_fail(log: &mut String) -> CompileResult {
    CompileResult { data: Err(()), to_log: log.to_string() }
}

/// Get the path of a binary in the provied LLVM directory.
fn get_compile_bin(bin: &str) -> PathBuf {
    let dir = PathBuf::from_str(BIN).unwrap();
    return dir.join(bin);
}

/// Format headers using the -I format.
fn format_headers(headers: &Vec<PathBuf>) -> Vec<String> {
    headers.iter()
           .map(|h| format!("-I{}", h.to_str().unwrap()))
           .collect()
}

fn compile_file(input: &CompileInput, log: &mut String) -> Result<String, ()> {
    // Get the path to clang from the args
    let clang = get_compile_bin("clang");
    let headers = format_headers(input.headers);

    // Run a quick compilation so we can check for errors
    let compile = Command::new("timeout")
        .arg("10")
        .arg(clang)
        .args(["-g", "-O3", "-emit-llvm", "-S"])
        .args(["-mllvm", "--enable-kernel-replacer"])
        .args(["-mllvm", "--gemmfarer-replacement-mode=cblas-interface"])
        .args(["-mllvm", "--debug-only=gemm-replacer-pass"])
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
    if compile.status.success() {
        log.push_str("success\n")
    } else {
        log.push_str("failed\n")
    }

    return Ok(output);
}

fn parse_output(input: &str) -> Vec<KernelMatch> {
    let mut acc = vec![];

    for (_, [line, kind]) in MATCH.captures_iter(input).map(|c| c.extract()) {
        let l = line.parse::<i64>().unwrap();
        let k = kind.parse::<i64>().unwrap();
        acc.push(KernelMatch { line: l, kind: k });
    }

    return acc;
}

/// Try to compile a file, & return the match data if successful.
pub fn try_compile(input: &CompileInput, log: &mut String) -> CompileResult {
    // Compile the file
    let out = match compile_file(input, log) {
        Ok(src) => src,
        Err(_) => return compile_fail(log),
    };

    // Parse the results
    let data = parse_output(&out);
    let m = Match { file: input.file.into(), data};
    CompileResult { data: Ok(Box::new(m)), to_log: log.to_string() }
}
