use std::collections::HashMap;
use std::path::PathBuf;

use crate::loops::Loops;

/// Parsed data from the `-debug-only` output.
pub type DebugInfo = HashMap<(i64, i64), SIStatus>;

/// SI status.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum SIStatus {
    FloatingPoint,  // Not allowed because of FP instructions
    ControlFlow,    // Not allowed becuase of control flow
    Enabled,        // SI is enabled & allowed. SI will be non zero if cost-effective
    Disabled,       // Not enabled for this loop
}

#[derive(Debug)]
pub struct Remark {
    pub vector: i64,
    pub width: i64,
    pub si: i64,
}

/// Communication between the compile & intern phases.
#[derive(Debug)]
pub struct Match {
    pub file: PathBuf,
    pub output: String,
    pub loops: Loops,
}

