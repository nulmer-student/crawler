use std::panic;

use crate::config::Config;
use crate::miner::mine;
use super::db;
use super::git::RepoData;

use rayon::iter::IntoParallelRefIterator;
use rayon::{current_thread_index, prelude::*, ThreadPool};
use sqlx::{self, Row};
use sqlx::any::AnyRow;
use log::{info, error};

// =============================================================================
// Top-Level Runner
// =============================================================================

pub fn run_all(config: &Config) {
    let db = db::Database::new(config);

    // Get all un-mined repos
    let repos = db.rt.block_on(un_mined_repos(&db))
                     .expect("Failed to fetch repos");

    // Create the runner thread pool
    info!("Creating runner thread pool");
    let run_pool = rayon::ThreadPoolBuilder::new()
        .num_threads(config.runner.threads)
        .thread_name(|i| { format!("run-{}", i) })
        .build()
        .expect("Failed to create runner thread pool");

    // Create miner thread pools
    info!("Creating miner thread pools");
    let miner_pools: Vec<_> = (0..config.runner.threads)
        .map(|i| {
            rayon::ThreadPoolBuilder::new()
                .num_threads(config.miner.threads)
                .thread_name(move |ii| format!("mine-{}:{}", i, ii))
                .build()
                .expect("Failed to create miner thread pool")
        })
        .collect();

    // Mine all repos
    run_pool.install(|| {
        let _ = repos.par_iter().for_each(|repo| {
            info!("Before");
            let pool = &miner_pools[current_thread_index().unwrap()];
            let mut runner = Runner::new(config, pool, repo.clone());
            runner.run();
            info!("After");
        });
    })
}

async fn un_mined_repos(db: &db::Database) -> Result<Vec<RepoData>, sqlx::Error> {
    // Fetch the results
    let rows = sqlx::query(
        "select *
         from repos
         where repo_id not in (select repo_id from mined)
        ").fetch_all(&db.pool).await?;

    let mut acc = vec![];
    for row in rows {
        let row = RepoData::from_row(row)?;
        acc.push(row);
    }

    return Ok(acc);
}

// =============================================================================
// Single Runner
// =============================================================================

struct Runner<'a> {
    config: &'a Config,
    pool: &'a ThreadPool,
    repo: RepoData,
}

impl<'a> Runner<'a> {
    /// Create a new runner
    pub fn new(config: &'a Config, pool: &'a ThreadPool, repo: RepoData) -> Self {
        return Self { config, pool, repo };
    }

    /// Mine this repo
    pub fn run(&mut self) {
        // Clone the repository
        let _ = self.repo.git_clone(&self.config.runner.tmp_dir);

        // Run the miner
        let dir = (&self.repo.dir).clone().unwrap(); // Dir is cloned
        let result = mine(&dir, self.config, self.pool);

        // if result.is_err() {
        //     error!("Failed mining: '{}'", self.repo.name);
        // }

        // Set this repo as mined
        info!("Finished mining: '{}'", self.repo.name);
    }
}
