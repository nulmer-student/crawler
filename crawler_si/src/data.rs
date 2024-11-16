use std::collections::HashMap;
use std::path::PathBuf;

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

/// Communication between the compile & intern phases.
#[derive(Debug)]
pub struct Match {
    pub file: PathBuf,
    pub output: String,
}

/// Values returned from the loop information pass
#[derive(Debug)]
#[allow(dead_code)]
pub struct LoopInfo {
    pub line: i64,
    pub col: i64,
    pub ir_count: i64,
    pub ir_mem: i64,
    pub ir_arith: i64,
    pub ir_other: i64,
    pub pat_start: Option<i64>,
    pub pat_step: Option<i64>,
}
