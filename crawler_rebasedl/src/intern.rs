use crawler::interface::{InternInput, InternResult};

use std::path::PathBuf;
use log::{error, warn};
use sqlx::{self, Row, Transaction};
use sqlx::Any;

use crate::data::{Match, PackingCandidate};

pub fn intern_matches(conn: &mut Transaction<'_, Any>, input: InternInput) -> InternResult {
    for file_data in input.data {
        let Some(data) = file_data.downcast_ref::<Match>() else {
            continue;
        };

        // Add the file
        let file_id = input.db.rt.block_on(ensure_file(
            conn, &data.file, input.repo_id
        ));
        let file_id = match file_id {
            Ok(id) => id,
            Err(e) => {
                error!("Failed to ensure file: {:?}", e);
                continue;
            }
        };

        // Insert each match
        for candidate in &data.data {
            let match_id = input.db.rt.block_on(new_match_id(conn));
            let match_id = match match_id {
                Ok(id) => id,
                Err(e) => {
                    error!("Failed to get match id: {:?}", e);
                    continue;
                }
            };
            if let Err(e) = input.db.rt.block_on(
                insert_match(conn, match_id, file_id, candidate)
            ) {
                error!("Failed to insert match: {:?}", e);
                continue;
            }
        }
    }

    return Ok(());
}

/// Get the file_id of FILE.
async fn file_id(pool: &mut Transaction<'_, Any>, file: &PathBuf, repo: i64) -> Option<i64> {
    let row = sqlx::query::<Any>(
        "select file_id
         from files
         where repo_id = ? and path = ?"
    ).bind(repo)
     .bind(file.to_str())
     .fetch_one(pool.as_mut())
     .await;

    match row {
        Ok(row) => Some(row.get::<i64, usize>(0)),
        Err(_) => None,
    }
}

/// Ensure that the given file exists in the database.
async fn ensure_file(conn: &mut Transaction<'_, Any>, file: &PathBuf, repo: i64) -> Result<i64, sqlx::Error> {
    match file_id(conn, file, repo).await {
        Some(id) => {
            Ok(id)
        }
        None => {
            // Insert the file
            sqlx::query::<Any>(
                "insert into files values (uuid_short(), ?, ?)"
            )
                .bind(repo)
                .bind(file.to_str())
                .execute(conn.as_mut())
                .await?;

            return Ok(file_id(conn, file, repo).await.unwrap());
        }
    }
}

/// Return a new match id.
async fn new_match_id(conn: &mut Transaction<'_, Any>) -> Result<i64, sqlx::Error> {
    let row = sqlx::query::<Any>("select uuid_short()")
        .fetch_one(conn.as_mut())
        .await;

    let id = match row {
        Ok(id) => Ok(id.get::<i64, usize>(0)),
        Err(e) => Err(e),
    };

    return id;
}

/// Insert a match into the database.
async fn insert_match(conn: &mut Transaction<'_, Any>, match_id: i64, file_id: i64, data: &PackingCandidate) -> Result<(), sqlx::Error> {
    sqlx::query::<Any>("insert into matches values (?, ?, ?, ?, ?, ?, ?)")
        .bind(match_id)
        .bind(file_id)
        .bind(data.line)
        .bind(data.column)
        .bind(data.min_access_frequency)
        .bind(data.cache_utilization)
        .bind(data.cost_benefit)
        .execute(conn.as_mut())
        .await?;

    return Ok(());
}
