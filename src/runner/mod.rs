mod db;
mod git;
mod search;
mod run;

use crate::config::Config;
use search::Search;
use run::run_all;

pub fn crawl(config: &Config) {
    // Search for matching repositories
    search(config);

    // Mine each repository
    run_all(config);
}

pub fn search(config: &Config) {
    let db = db::Database::new(config);
    let _search = Search::new(config, &db);
}
