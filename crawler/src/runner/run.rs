use crate::config::Config;
use crate::interface::{AnyInterface, InitInput, InternInput};
use crate::miner::{mine, MineResult};
use super::db;
use super::git::RepoData;

use rayon::iter::IntoParallelRefIterator;
use rayon::{current_thread_index, prelude::*, ThreadPool};
use sqlx::{self, Any};
use log::{info, error};
use crossbeam::sync::WaitGroup;
use std::panic::AssertUnwindSafe;
use std::sync::mpsc;
use std::time::Instant;

// =============================================================================
// Top-Level Runner
// =============================================================================

pub fn run_all(config: &Config, interface: AnyInterface) {
    let db = db::Database::new(config);

    // Call the user supplied init function
    info!("Initializing instance");
    let input = InitInput { config, db: &db };
    match interface.init(input) {
        Ok(_) => {},
        Err(e) => { panic!("Failed to initialize instance: {:?}", e) }
    }

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
    info!("Mining {} repositories", repos.len());
    run_pool.install(|| {
        let _ = repos.par_iter().for_each(|repo| {
            // Mine a single repo
            let result = std::panic::catch_unwind(AssertUnwindSafe(|| {
                let pool = &miner_pools[current_thread_index().unwrap()];
                let mut runner = Runner::new(config, pool, &db, repo.clone(), interface.clone());
                runner.run();
            }));

            // Error out if there is a panic
            if let Err(_) = result {
                error!("Runner paniced")
            }
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
    db: &'a db::Database,
    repo: RepoData,
    start: Instant,
    interface: AnyInterface,
}

impl<'a> Runner<'a> {
    /// Create a new runner
    pub fn new(config: &'a Config, pool: &'a ThreadPool, db: &'a db::Database, repo: RepoData, interface: AnyInterface) -> Self {
        return Self { config, pool, db, repo, start: Instant::now(), interface };
    }

    /// Mine this repo
    pub fn run(&mut self) {
        // Clone the repository
        let _ = self.repo.git_clone(&self.config.runner.tmp_dir);

        // If we don't have the WaitGroup, the current thread continues on and
        // deletes the repo before we have mined it.
        let wg = WaitGroup::new();

        let (tx, rx) = mpsc::channel::<MineResult>();

        // Run the miner using our thread pool
        if let Some(dir) = (&self.repo.dir).clone() {
            // Get the name of the log file
            let log_file = self.repo.name.replace("/", "-");
            let log_file = format!("{}-{}.log", self.repo.id, log_file);
            let log_path = self.config.runner.log_dir.join(log_file);

            let config = self.config.clone();
            let wg = wg.clone();
            let interface = self.interface.clone();
            self.pool.spawn(move || {
                // Run the miner
                if let Ok(result) = mine(&dir, &log_path, config, interface) {
                    match tx.send(result) {
                        Ok(_) => {},
                        Err(e) => {
                            error!("Failed to send match data: {}", e);
                        }
                    }
                }
                drop(wg);
            });
        }
        wg.wait();

        // Add any matches
        let result = match rx.try_recv() {
            Ok(r) => r,
            Err(e) => {
                error!("Failed to receive match data: {:?}", e);
                return;
            }
        };

        // Setup the input
        info!("Interning results");
        let input = InternInput {
            config: self.config,
            repo_id: self.repo.id,
            data: &result.data,
            db: self.db,
        };

        // Call the user-supplied intern function
        match self.interface.intern(input) {
            Ok(_) => {},
            Err(e) => error!("Failed to intern: {:?}", e),
        }

        self.db.rt.block_on(self.mark_as_mined(&result));
        info!("Finished mining: '{}'", self.repo.name);
    }

    /// Mark the current repository as mined.
    async fn mark_as_mined(&self, data: &MineResult) {
        // Set as mined
        let repo_id = self.repo.id;
        let result = sqlx::query::<Any>("insert into mined values (?)")
            .bind(repo_id)
            .execute(&self.db.pool)
            .await;

        match result {
            Ok(_) => {},
            Err(e) => { error!("Failed to set repo as mined: {:?}", e) },
        }

        // Insert the statistics
        let time = format!("{}", self.start.elapsed().as_millis());
        let result = sqlx::query::<Any>("insert into stats values (?, ?, ?, ?, ?)")
            .bind(repo_id)
            .bind(data.n_files)
            .bind(data.n_success)
            .bind(data.n_error)
            .bind(time)
            .execute(&self.db.pool)
            .await;

        match result {
            Ok(_) => {},
            Err(e) => { error!("Failed to add statistics: {:?}", e) },
        }
    }
}

impl Drop for Runner<'_> {
    fn drop(&mut self) {
        info!("Drop runner");
    }
}
