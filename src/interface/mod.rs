mod find_vector_si;

use std::path::PathBuf;
use std::fs;
use std::sync::Arc;

pub type PreprocessResult = Result<String, ()>;
pub type CompileResult    = Result<String, ()>;
pub type InternResult     = Result<(), ()>;

pub trait Interface {
    /// Called once on the source file, the result is sent to the compile phase.
    /// By default, returns the file contents.
    fn preprocess(&self, _root: &PathBuf, file: &PathBuf) -> PreprocessResult {
        match fs::read_to_string(file) {
            Ok(s) => Ok(s),
            Err(_) => Err(()),
        }
    }

    /// Compile FILE with HEADERS. Returns Ok() if the compilation succeeds and
    /// Err() otherwise. The resultant string is sent to the interning phase.
    fn compile(
        &self,
        content: &str,
        root: &PathBuf,
        file: &PathBuf,
        headers: &Vec<PathBuf>
    ) -> CompileResult;

    /// Given CONTENT, add any possible matches to the database.
    fn intern(&self, content: &str) -> InternResult;
}


pub fn get_interface(name: &str) -> Arc<dyn Interface + Send + Sync> {
    match name {
        "si" => {
            Arc::new(find_vector_si::FindVectorSI {}) as Arc<dyn Interface + Send + Sync>
        },
        _ => { panic!("No interface with name: '{}'", name) },
    }
}
