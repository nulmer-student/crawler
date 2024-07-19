use super::{Interface, InternResult, CompileResult};

use std::path::PathBuf;

pub struct FindVectorSI {}

impl Interface for FindVectorSI {
    fn compile(
        &self,
        _content: &str,
        _root: &PathBuf,
        _file: &PathBuf,
        _headers: &Vec<PathBuf>
    ) -> CompileResult {
        return Err(());
    }

    fn intern(&self, _content: &str) -> InternResult {
        return Err(());
    }
}
