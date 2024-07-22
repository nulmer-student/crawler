use crate::config::Config;
use super::db::Database;

use sqlx;

pub struct Search<'a> {
    config: &'a Config,
    db: &'a Database,
}


impl<'a> Search<'a> {
    /// Create a new searcher
    pub fn new(config: &'a Config, db: &'a Database) -> Self {
        let search = Self { config, db };
        search.search();
        return search;
    }

    /// Search for all repos matching the search criteria.
    pub fn search(&self) {

    }
}

// pub async fn search(db: &Database, config: &Config) -> Result<(), sqlx::Error> {
//     return Ok(());
// }
