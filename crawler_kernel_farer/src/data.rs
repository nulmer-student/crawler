use std::path::PathBuf;

#[derive(Debug, Default)]
pub struct KernelMatch {
    pub line: i64,
    pub kind: i64,
}

pub struct Match {
    pub file: PathBuf,
    pub data: Vec<KernelMatch>,
}
