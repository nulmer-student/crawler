use std::{path::PathBuf, process::Command};
use std::fs;

use serde_json::value::Value;
use sqlx::{any::AnyRow, Row};
use log::{error, info, warn};

#[derive(Clone, Debug)]
pub struct RepoData {
    // Repository info
    pub id: i64,        // The Any db backend requires signed integers
    pub name: String,
    pub url: String,
    pub stars: i64,

    // Physical repo
    pub dir: Option<PathBuf>,
}

impl RepoData {
    /// Create a repo from JSON.
    pub fn from_json(data: &Value) -> Result<Self, ()> {
        let id    = &data["id"].as_i64().ok_or(())?;
        let name  = &data["full_name"].as_str().ok_or(())?;
        let url   = &data["clone_url"].as_str().ok_or(())?;
        let stars = &data["stargazers_count"].as_i64().ok_or(())?;

        return Ok(Self {
            id: *id,
            name: name.to_string(),
            url: url.to_string(),
            stars: *stars,
            dir: None,
        });
    }

    /// Create a repo from a database row.
    pub fn from_row(row: AnyRow) -> Result<Self, sqlx::Error> {
        let id    = row.try_get::<i64, usize>(0)?;
        let name  = row.try_get::<&[u8], usize>(1)?;
        let url   = row.try_get::<&[u8], usize>(2)?;
        let stars = row.try_get::<i64, usize>(3)?;

        let name = String::from_utf8(name.to_vec()).unwrap();
        let url  = String::from_utf8(url.to_vec()).unwrap();

        return Ok(Self { id, name, url, stars, dir: None });
    }

    /// Clone this repo and return the directory cloned to.
    pub fn git_clone(&mut self, tmp_dir: &PathBuf) -> Result<(), String> {
        info!("Starting clone of '{}'", self.name);

        // If the directory already exists, delete what is there
        let dir = tmp_dir.join(format!("{}", self.id));
        if dir.exists() {
            warn!("Removing pre-existing files at: {:?}", dir);
            let _ = fs::remove_dir_all(&dir);
        }

        // Clone the repo
        let out = Command::new("git")
            .arg("clone")
            .arg(&self.url)
            .arg("--depth")
            .arg("1")
            .arg(&dir)
            .output()
            .expect("Failed to execute git");

        // Error if there is a non-zero exit code
        if out.status.success() {
            info!("Finished cloning '{}' to {:?}", self.name, dir);
            self.dir = Some(dir);
            return Ok(());
        } else {
            let err = String::from_utf8(out.stderr).unwrap();
            error!("Failed to clone '{}': {}", self.name, err);
            return Err(err);
        }
    }
}

/// Delete the Repository files when the repo goes out of scope
impl Drop for RepoData {
    fn drop(&mut self) {
        if let Some(dir) = &self.dir {
            info!("Deleting repo: '{}' at {:?}", self.name, dir);
            let _ = fs::remove_dir_all(dir);
        }
    }
}
