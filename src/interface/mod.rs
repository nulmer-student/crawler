mod find_vector_si;

use crate::config::Config;

use std::any::Any;
use std::path::PathBuf;
use std::fs;
use std::sync::Arc;
use log::error;

pub type MatchData = Box<dyn Any + Send + Sync>;

#[allow(dead_code)]
pub struct PreInput<'a> {
    pub config: &'a Config,
    pub root: &'a PathBuf,
    pub file: &'a PathBuf,
}

pub type PreprocessResult = Result<String, ()>;

#[allow(dead_code)]
pub struct CompileInput<'a> {
    pub config: &'a Config,
    pub root: &'a PathBuf,
    pub file: &'a PathBuf,
    pub content: &'a str,           // File after preprocessing
    pub headers: &'a Vec<PathBuf>   // Header choices
}

pub type CompileResult = Result<MatchData, ()>;

#[allow(dead_code)]
pub struct InternInput<'a> {
    pub config: &'a Config,
    pub root: &'a PathBuf,
    pub file: &'a PathBuf,
    pub data: &'a MatchData,
}

pub type InternResult = Result<(), ()>;

pub trait Interface {
    /// Called once on the source file, the result is sent to the compile phase.
    /// By default, returns the file contents.
    fn preprocess(&self, input: PreInput) -> PreprocessResult {
        match fs::read_to_string(input.file) {
            Ok(s) => Ok(s),
            Err(e) => {
                error!("Failed to read file: {:?}", e);
                Err(())
            },
        }
    }

    /// Called for each source file. If this returns Ok, the results are pased
    /// to the intern phase. Otherwise, alternative headers are tried.
    fn compile(&self, input: CompileInput) -> CompileResult;

    /// Called after all mining has finished with any compilation results.
    /// Intended for adding matches to the database.
    fn intern(&self, input: InternInput) -> InternResult;
}


pub fn get_interface(name: &str) -> Arc<dyn Interface + Send + Sync> {
    match name {
        "si" => {
            Arc::new(find_vector_si::FindVectorSI {}) as Arc<dyn Interface + Send + Sync>
        },
        _ => { panic!("No interface with name: '{}'", name) },
    }
}
