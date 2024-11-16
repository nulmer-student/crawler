use crate::config::Config;
use crate::runner::db;

use std::any::Any;
use std::panic::RefUnwindSafe;
use std::path::PathBuf;
use std::fs;
use std::sync::Arc;
use log::error;

pub type MatchData = Box<dyn Any + Send + Sync>;

// Initialization:

#[allow(dead_code)]
pub struct InitInput<'a> {
    pub config: &'a Config,
    pub db: &'a db::Database,
}

pub type InitResult = Result<(), String>;

// Preprocessing:

#[allow(dead_code)]
pub struct PreInput<'a> {
    pub config: &'a Config,
    pub root: &'a PathBuf,
    pub file: &'a PathBuf,
}

pub type PreprocessResult = Result<String, ()>;

// Compilation:

#[allow(dead_code)]
pub struct CompileInput<'a> {
    pub config: &'a Config,
    pub root: &'a PathBuf,
    pub file: &'a PathBuf,
    pub content: &'a str,           // File after preprocessing
    pub headers: &'a Vec<PathBuf>   // Header choices
}

pub struct CompileResult {
    pub data: Result<MatchData, ()>,    // Instance specific match data
    pub to_log: String,                 // Data to output to the current repositories log
}

// Intern:

#[allow(dead_code)]
#[derive(Clone)]
pub struct InternInput<'a> {
    pub config: &'a Config,
    pub repo_id: i64,
    pub data: &'a Vec<MatchData>,
    pub db: &'a db::Database,
}

pub type InternResult = Result<(), ()>;

pub type AnyInterface = Arc<dyn Interface + Sync + Send + RefUnwindSafe>;

pub trait Interface {
    /// Called once after the search has finished but before any preprocessing /
    /// compilation happens. Does nothing by default.
    fn init(&self, _input: InitInput) -> InitResult {
        return Ok(());
    }

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
