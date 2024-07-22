mod db;
mod search;

use crate::config::Config;
use search::Search;

pub fn crawl(config: &Config) {
    // Search for matching repositories
    search(config);

    // Mine each repository
    // TODO
}

pub fn search(config: &Config) {
    let db = db::Database::new(config);
    let _search = Search::new(config, &db);
}
