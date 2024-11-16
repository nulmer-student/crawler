use cmake::Config;
use std::{env, path::PathBuf};

const LLVM_DIR: &str = "LLVM_INSTALL_DIR";

fn depend_on(path: &str) {
    println!("cargo::rerun-if-changed={path}");
}

fn depend_on_env(var: &str) {
    println!("cargo::rerun-if-env-changed={var}");
}

fn export_env(var: &str, value: PathBuf) {
    println!("cargo::rustc-env={var}={}", value.to_str().unwrap());
}

macro_rules! echo {
    ($($tokens: tt)*) => {
        println!("cargo:warning={}", format!($($tokens)*))
    };
}

fn main() {
    // Re run if anything changes
    depend_on("passes/find_inner_loops/FindInnerLoops.cpp");
    depend_on("passes/find_inner_loops/FindInnerLoops.h");
    depend_on("passes/information/Information.cpp");
    depend_on("passes/information/Information.h");
    depend_on_env(LLVM_DIR);

    // Build the required LLVM passes
    let llvm_dir = env::var(LLVM_DIR).unwrap();
    echo!("Building LLVM passes using: {llvm_dir}");
    let dst = Config::new("passes")
        .configure_arg(&format!("-DLT_LLVM_INSTALL_DIR={llvm_dir}"))
        .build_target("all")
        .build();
    println!("cargo:rustc-link-search=native={}", dst.display());

    // Create environment variables for the pass binaries
    let bin_dir = dst.as_path().join("build/lib");
    export_env("CRAWLER_SI_LOOPS", bin_dir.join("libFindInnerLoops.so"));
    export_env("CRAWLER_SI_INFO", bin_dir.join("libInformation.so"));
    export_env("CRAWLER_SI_LLVM", PathBuf::from(llvm_dir).join("bin"));
}
