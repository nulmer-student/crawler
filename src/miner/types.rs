use std::path::{Components, PathBuf};

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
    pub fn _relative(string: &str, directory: &PathBuf) -> Self {
        let path: PathBuf = PathBuf::from(string)
            .strip_prefix(directory)
            .unwrap()
            .to_path_buf();

        return File::new(path);
    }

    /// Return an iterator of the components of a file
    pub fn components(&self) -> Components {
        return self.path.components();
    }

    pub fn kind(&self) -> FileType {
        return self.kind.clone();
    }

    pub fn path(&self) -> &PathBuf {
        return &self.path;
    }
}

// =============================================================================
// Include Declaration
// =============================================================================

/// Type of a header, either user or system.
#[derive(Debug, Hash)]
pub enum DeclareType {
    User,       // #include "file.h"
    System,     // #include <file.h>
}

/// Type representing an include declaration
#[derive(Debug, Hash)]
pub struct Declare {
    kind: DeclareType,
    path: PathBuf,      // Path in the declaration, not in the referenced file
}
