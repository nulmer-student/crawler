use std::path::PathBuf;

#[derive(Debug, Default)]
pub struct PackingCandidate {
    pub line: Option<i64>,
    pub column: Option<i64>,
    pub depth: Option<i64>,
    pub min_access_frequency: Option<f32>,
    pub cache_utilization: Option<f32>,
    pub cost_benefit: Option<f32>,
}

pub struct Match {
    pub file: PathBuf,
    pub data: Vec<PackingCandidate>,
}
