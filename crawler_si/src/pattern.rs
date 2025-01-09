use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    /// Parse the vectorization remark output.
    pub static ref MATCH_PATTERN: Regex = {
        let mut pattern = "".to_string();
        pattern.push_str(r"(\d+):(\d+): remark: vectorized loop \(");
        pattern.push_str(r"vectorization width: (\d+),");
        pattern.push_str(r" interleaved count: (\d+),");
        pattern.push_str(r" scalar interpolation count: (\d+)");
        pattern.push_str(r"\)");
        Regex::new(&pattern).unwrap()
    };

    /// Check if a contains a for loop.
    pub static ref LOOP_PATTERN: Regex = {
        Regex::new(r"for").unwrap()
    };

    pub static ref INFO_PATTERN: Regex = {
        let mut pattern = "".to_string();
        pattern.push_str(r"loop info: \[([^\]]+)\] \(");
        pattern.push_str(r"ir_count: (\d+), ");
        pattern.push_str(r"ir_mem: (\d+), ");
        pattern.push_str(r"ir_arith: (\d+), ");
        pattern.push_str(r"ir_other: (\d+), ");
        pattern.push_str(r"pat_start: (\S+), ");
        pattern.push_str(r"pat_step: (\S+)");
        pattern.push_str(r"\)");
        Regex::new(&pattern).unwrap()
    };
}

pub static PRAGMA: &str = "#pragma clang loop scalar_interpolation(enable)\n";
