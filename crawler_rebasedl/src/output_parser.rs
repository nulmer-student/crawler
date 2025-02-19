use crate::data::PackingCandidate;
use regex::Regex;
use lazy_static::lazy_static;

const DATA_START: &str = "[RebaseDLPass] RegionPackingCandidate ===========";
const DATA_END: &str = "[RebaseDLPass] ==================================";

lazy_static! {
    pub static ref LOC: Regex = {
        let mut pattern = "".to_string();
        pattern.push_str(r"!DILocation\(line: (\d+), column: (\d+)");
        Regex::new(&pattern).unwrap()
    };
    pub static ref DEPTH: Regex = {
        let mut pattern = "".to_string();
        pattern.push_str(r"- depth: (\d+)");
        Regex::new(&pattern).unwrap()
    };
    pub static ref MAF: Regex = {
        let mut pattern = "".to_string();
        pattern.push_str(r"Minimum access frequency: ([.0-9]+)");
        Regex::new(&pattern).unwrap()
    };
    pub static ref CU: Regex = {
        let mut pattern = "".to_string();
        pattern.push_str(r"Cache utilization: ([.0-9]+)");
        Regex::new(&pattern).unwrap()
    };
    pub static ref CB: Regex = {
        let mut pattern = "".to_string();
        pattern.push_str(r"Cost benefit: ([.0-9]+)");
        Regex::new(&pattern).unwrap()
    };
}

fn matching_lines(lines: &Vec<&str>, value: &str) -> Vec<usize> {
    lines.iter()
         .enumerate()
         .filter(|(_i, l)| **l == value)
         .map(|(i, _l)| i)
         .collect::<Vec<_>>()
}

pub fn parse(input: String) -> Vec<PackingCandidate> {
    // Compute the range of each data output
    let lines = input.lines().collect::<Vec<_>>();
    let starts = matching_lines(&lines, DATA_START);
    let ends = matching_lines(&lines, DATA_END);
    let ranges = starts.iter().zip(ends.iter()).collect::<Vec<_>>();


    // Parse the output
    let mut acc = vec![];
    for (start, end) in ranges {
        let mut candidate = PackingCandidate::default();

        let loc_line = lines[start + 2];
        if let Some(m) = LOC.captures(loc_line) {
            let line = m.get(1).unwrap().as_str().parse::<i64>().unwrap();
            let column = m.get(2).unwrap().as_str().parse::<i64>().unwrap();
            candidate.line = Some(line);
            candidate.column = Some(column);
        }

        let all = &lines[*start..*end].concat();
        if let Some(m) = DEPTH.captures(&all) {
            let depth = m.get(1).unwrap().as_str().parse::<i64>().unwrap();
            candidate.depth = Some(depth);
        }

        if let Some (m) = MAF.captures(&all) {
            let freq = m.get(1).unwrap().as_str().parse::<f32>().unwrap();
            candidate.min_access_frequency = Some(freq);
        }

        if let Some (m) = CU.captures(&all) {
            let util = m.get(1).unwrap().as_str().parse::<f32>().unwrap();
            candidate.cache_utilization = Some(util);
        }

        if let Some (m) = CB.captures(&all) {
            let benefit = m.get(1).unwrap().as_str().parse::<f32>().unwrap();
            candidate.cost_benefit = Some(benefit);
        }

        acc.push(candidate);
    }

    return acc;
}
