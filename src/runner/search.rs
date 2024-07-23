use std::str::FromStr;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::config::Config;
use super::db::Database;

use sqlx::{self, Row, Any};
use reqwest;
use reqwest::blocking::Client;
use reqwest::header;
use serde_json;
use serde_json::value::Value;

static USER_AGENT: &str = concat!(
    env!("CARGO_PKG_NAME"),
    "/",
    env!("CARGO_PKG_VERSION"),
);

static PAGE_SIZE: usize = 100;
static INITIAL_MAX: usize = 10_000_000;

#[derive(Debug)]
struct RepoData {
    pub id: i64,        // The Any db backend requires signed integers
    pub name: String,
    pub url: String,
    pub stars: i64,
}

impl RepoData {
    fn new(data: &Value) -> Result<Self, ()> {
        let id    = &data["id"].as_i64().ok_or(())?;
        let name  = &data["full_name"].as_str().ok_or(())?;
        let url   = &data["clone_url"].as_str().ok_or(())?;
        let stars = &data["stargazers_count"].as_i64().ok_or(())?;

        return Ok(Self {
            id: *id,
            name: name.to_string(),
            url: url.to_string(),
            stars: *stars,
        });
    }
}

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

        let min = self.config.runner.min_stars;
        let mut max = INITIAL_MAX;

        loop {
            println!("Found: '{}'", found);
            // Get the next page of results
            let (json, next_page) = self.get_page(min, max, page);

            // Move to the next page
            if next_page {
                page = 1;
                max = self.db.rt.block_on(self.min_stars());
            }

            // Otherwise, add the repositories to the database
            else {
                // Add the repos to the database
                let repos = self.parse_results(json);
                found += self.add_repos(repos);

                // Move to the next page
                page += 1;
            }

            // Stop running when the star interval is empty
            if max <= min {
                break;
            }
        }
    }

    /// Parse JSON into RepoData.
    fn parse_results(&self, json: Value) -> Vec<RepoData> {
        let mut acc = vec![];
        if let Value::Array(items) = &json["items"] {
            for item in items {
                let repo = match RepoData::new(item) {
                    Ok(r) => r,
                    Err(_) => panic!("Failed to parse repo JSON"),
                };
                acc.push(repo);
            }
        }
        return acc;
    }

    /// Add REPOS to the database. Ignore if they already exist.
    fn add_repos(&self, repos: Vec<RepoData>) -> usize {
        let mut count = 0;  // # of repos we added

        for repo in repos {
            println!("Adding repo: {:?}", repo);
            match self.db.rt.block_on(self.add_repo(repo)) {
                Ok(_) => { count += 1 },
                Err(e) => {
                    println!("Error inserting repository: '{}'", e);
                }
            }
        }

        return count;
    }

    /// Add a single repository to the database.
    async fn add_repo(&self, repo: RepoData) -> Result<(), sqlx::Error> {
        sqlx::query::<Any>("insert into repos values (?, ?, ?, ?)")
            .bind(repo.id)
            .bind(repo.name)
            .bind(repo.url)
            .bind(repo.stars)
            .execute(&self.db.pool)
            .await?;

        Ok(())
    }

    /// Get a single page of results.
    fn get_page(&self, min: usize, max: usize, page_no: usize) -> (Value, bool) {
        // Get the next page
        let query = self.query(min, max, page_no);
        println!("{:#?}", query);
        let result = self.client.get(query)
                                .send()
                                .unwrap();

        // Perform any rate limiting
        self.rate_limit(result.headers());

        // Parse the results
        let json: Value = serde_json::from_str(&result.text().unwrap()).unwrap();

        // If the item key is present, we must go to the next page
        match json["items"] {
            Value::Null => {
                return (json, true);
            },
            _ => {
                return (json, false);
            }
        }
    }

    /// Perform any required rate-limiting.
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
    fn query(&self, min: usize, max: usize, page_no: usize) -> String {
        format!(
            "https://api.github.com/search/repositories?q=language:c+stars:{}..{}&{}&{}&per_page={}&page={}",
            min,
            max,
            "sort=stars",
            "order=desc",
            PAGE_SIZE,
            page_no,
        )
    }

    /// Return the minimum star count of any repository.
    async fn min_stars(&self) -> usize {
        let row = sqlx::query("select min(stars) from repos")
            .fetch_one(&self.db.pool)
            .await;

        match row {
            Ok(row) => {
                return row.get::<i64, usize>(0) as usize;
            },
            Err(_) => {
                return INITIAL_MAX;
            }
        }
    }
}
