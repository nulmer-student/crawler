use log::debug;

use super::{CompileInput, CompileResult, Interface, InternResult, InternInput};
use std::{io::{BufReader, BufWriter, Read, Write}, process::{Command, Stdio}};
use std::any::Any;

pub struct FindVectorSI {}

impl Interface for FindVectorSI {
    fn compile(&self, input: CompileInput) -> CompileResult {
        // Get the path to clang from the args
        let clang = &input.config.interface.args["clang"];

        // Format the headers with "-I"
        let headers: Vec<_> = input
            .headers
            .iter()
            .map(|h| format!("-I{}", h.to_str().unwrap()))
            .collect();

        // Compilation command
        let mut cmd = Command::new(clang);
        let mut compile = cmd
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .arg("-c")
            .args(["-x", "c"])
            .args(headers)
            .args(["-o", "/dev/null"])
            .arg("-emit-llvm")
            .arg("-O3")
            .arg("-Rpass=loop-vectorize")
            .arg("-")
            .spawn()
            .unwrap();

        // Send the input source file
        let mut stdin = compile.stdin.take().unwrap();
        stdin.write_all(input.content.as_bytes()).unwrap();
        drop(stdin);    // Blocks if we don't have this

        // Get the compilation output
        let out = compile.wait_with_output().unwrap();

        // If the compilation was successful, return the stderr
        if out.status.success() {
            let result: Box<dyn Any> = Box::new(out.stderr);
            return Ok(result);
        }

        // Otherwise, error out
        return Err(());
    }

    fn intern(&self, _input: InternInput) -> InternResult {
        return Err(());
    }
}
