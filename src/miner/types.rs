use std::path::PathBuf;

// =============================================================================
// File
// =============================================================================

/// Type of a file, either a source file or a header.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum FileType {
    Source,
    Header,
}

/// Program file.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct File {
    kind: FileType,
    path: PathBuf,
}

impl File {
    /// Create a new file.
    pub fn new(path: PathBuf) -> Self {
        let Some(ext) = path.extension() else {
            panic!("Missing extension '{}'", path.display());
        };

        let kind = match ext.to_str().unwrap() {
            "c" => { FileType::Source },
            "h" => { FileType::Header },
            _ => { panic!("Unsupported file type: '{}'", path.display()) },
        };

        return File { path, kind };
    }

    /// Create a new file realtive to DIRECTORY.
    pub fn relative(string: &str, directory: &PathBuf) -> Self {
        let path: PathBuf = PathBuf::from(string)
            .strip_prefix(directory)
            .unwrap()
            .to_path_buf();

        return File::new(path);
    }
}

// =============================================================================
// Header
// =============================================================================

/// Type of a header, either user or system.
#[derive(Debug, Hash)]
pub enum HeaderType {
    User,       // #include "file.h"
    System,     // #include <file.h>
}

/// Type representing an include declaration
#[derive(Debug, Hash)]
pub struct Include {
    kind: HeaderType,
    path: PathBuf,
}
