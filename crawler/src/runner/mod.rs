pub mod db;
mod git;
mod search;
mod run;

use std::sync::Arc;

use crate::{config::Config, interface::{AnyInterface, Interface}};
use search::Search;
use run::run_all;

pub fn crawl(config: &Config, interface: AnyInterface) {
    // Search for matching repositories
    search(config);

    // Mine each repository
    run_all(config, interface);
}

pub fn search(config: &Config) {
    let db = db::Database::new(config);
    let _search = Search::new(config, &db);
}
