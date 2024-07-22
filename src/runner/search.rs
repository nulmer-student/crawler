use std::str::FromStr;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::config::Config;
use super::db::Database;

use sqlx;
use reqwest;
use reqwest::blocking::Client;
use reqwest::header;

static USER_AGENT: &str = concat!(
    env!("CARGO_PKG_NAME"),
    "/",
    env!("CARGO_PKG_VERSION"),
);

static PAGE_SIZE: usize = 100;
static BLOCK_SIZE: usize = 1000;

pub struct Search<'a> {
    config: &'a Config,
    db: &'a Database,

    client: Client,
}

impl<'a> Search<'a> {
    /// Search for repositories.
    pub fn new(config: &'a Config, db: &'a Database) -> Self {
        let mut headers = header::HeaderMap::new();
        headers.insert(
            "Authorization",
            header::HeaderValue::from_str(
                &format!("Bearer {}", config.runner.github_api_key)
            ).unwrap()
        );
        headers.insert(
            "X-GitHub-Api-Version",
            header::HeaderValue::from_static("2022-11-28")
        );

        let client = reqwest::blocking::Client::builder()
            .user_agent(USER_AGENT)
            .default_headers(headers)
            .build()
            .unwrap();

        // Run the search
        let search = Self { config, db, client };
        search.search();

        return search;
    }

    /// Search for all repos matching the search criteria.
    pub fn search(&self) {
        let mut found = 0;
        let mut page = 1;

        let min = 500;
        let max = 10_000_000;

        loop {
            // Get the next page of results
            self.get_page(min, page);

            found += PAGE_SIZE;
            page += 1;
        }
    }

    /// Get a single page of results.
    fn get_page(&self, min: usize, page_no: usize) {
        // Get the next page
        let result = self.client.get(self.query(page_no))
                                .send()
                                .unwrap();

        // Perform any rate limiting
        self.rate_limit(result.headers());

        // Parse the results
        // TODO

        println!("{:#?}", result);
    }

    fn rate_limit(&self, headers: &header::HeaderMap) {
        let mut time = 0.0;

        // If the "retry-after" header is present, sleep for the time it gives
        if headers.contains_key("retry-after") {
            time = self.from_header(headers, "retry-after");
        }

        // Try rate limit headers
        if self.from_header::<usize>(headers, "x-ratelimit-remaining") == 0 {
            let now = Self::parse_time(SystemTime::now());
            let then = self.from_header::<u64>(headers, "x-ratelimit-reset");
            time = (then - now + 1) as f64;
        }

        // Sleep if needed
        if time != 0.0 {
            println!("Sleeping for {}s", time);
            thread::sleep(Duration::from_secs_f64(time));
        }
    }

    /// Return TIME as an integer (unix time).
    fn parse_time(time: SystemTime) -> u64 {
        time.duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }

    /// Get a NAME from HEADERS & parse it to a given type
    fn from_header<T: FromStr>(&self, headers: &header::HeaderMap, name: &str) -> T {
        match headers[name].to_str().unwrap().parse::<T>() {
            Ok(s) => s,
            _ => panic!(),
        }
    }

    /// Format the query.
    fn query(&self, page_no: usize) -> String {
        format!(
            "https://api.github.com/search/repositories?{}&{}&{}&per_page={}&page={}",
            "q=language:c",
            "sort=stars",
            "order=desc",
            PAGE_SIZE,
            page_no,
        )
    }
}
