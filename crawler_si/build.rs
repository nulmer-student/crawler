use cmake::Config;

fn main() {
    // Build the required LLVM passes
    println!("Building LLVM passes");
    let dst = Config::new("passes")
        .configure_arg("-DLT_LLVM_INSTALL_DIR=/home/nju/.opt/scalar/llvm-bin/")
        .build_target("all")
        .build();

    println!("cargo:rustc-link-search=native={}", dst.display());
}
