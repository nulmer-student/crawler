use crawler::interface::{InternInput, InternResult};
use crate::data::{DebugInfo, Match, SIStatus};
use crate::loops::LoopInfo;
use crate::pattern::MATCH_PATTERN;

use std::path::PathBuf;
use log::{error, warn};
use sqlx::{self, Row, Transaction};
use sqlx::Any;

pub fn intern_matches(conn: &mut Transaction<'_, Any>, input: InternInput) -> InternResult {
    for m in input.data {
        if let Some(entry) = m.downcast_ref::<Match>() {
            // Parse the loop info
            let loop_info = vec!();

            // Parse the -debug-only
            // let debug_info = parse_vector_debug(&entry.output);
            let debug_info: DebugInfo = DebugInfo::new();

            // Parse the output for vectorization opps
            let pattern = &MATCH_PATTERN;
            for (_body, args) in pattern
                .captures_iter(&entry.output).map(|c| c.extract::<5>())
            {
                let args: [i64; 5] = args
                    .iter()
                    .map(|a| a.parse::<i64>().unwrap())
                    .collect::<Vec<i64>>()
                    .try_into()
                    .unwrap();
                let line = args[0];
                let col  = args[1];

                // Add the file to the files table
                let file_id = input.db.rt.block_on(ensure_file(
                    conn, &entry.file, input.repo_id
                ));
                let file_id = match file_id {
                    Ok(id) => id,
                    Err(e) => {
                        error!("Failed to ensure file: {:?}", e);
                        continue;
                    }
                };

                // Insert the match & location
                let match_id = input.db.rt.block_on(new_match_id(conn));
                let match_id = match match_id {
                    Ok(id) => id,
                    Err(e) => {
                        error!("Failed to get match id: {:?}", e);
                        continue;
                    }
                };
                if let Err(e) = input.db.rt.block_on(
                    insert_match(conn, match_id, file_id, line, col)
                ) {
                    error!("Failed to insert match: {:?}", e);
                    continue;
                }

                // Insert vector remarks
                if let Err(e) = input.db.rt.block_on(
                    insert_remarks(conn, match_id, args[2], args[3], args[4])
                ) {
                    error!("Failed to insert remarks: {:?}", e);
                    continue;
                }

                // Check to see if there is loop info for this loop
                if let Some(info) = find_loop_info(&loop_info, line, col) {
                    // Insert IR mix
                    if let Err(e) = input.db.rt.block_on(
                        insert_ir_mix(conn, match_id, info)
                    ) {
                        error!("Failed to insert ir mix: {:?}", e);
                        continue;
                    }

                    // Insert loop pattern
                    if let Err(e) = input.db.rt.block_on(
                        insert_mem_pattern(conn, match_id, info)
                    ) {
                        error!("Failed to insert loop pattern: {:?}", e);
                        continue;
                    }
                }

                // Check to see if there is debug info
                if let Some(info) = debug_info.get(&(line, col)) {
                    if let Err(e) = input.db.rt.block_on(
                        insert_si_status(conn, match_id, &info)
                    ) {
                        error!("Failed to insert debug info: {:?}", e);
                    }
                } else {
                    warn!(
                        "Failed to find debug info for {:?} in {:?}",
                        (line, col), entry.file
                    )
                }
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
async fn insert_match(conn: &mut Transaction<'_, Any>, match_id: i64, file_id: i64, line: i64, col: i64) -> Result<(), sqlx::Error> {
    sqlx::query::<Any>("insert into matches values (?, ?, ?, ?)")
        .bind(match_id)
        .bind(file_id)
        .bind(line)
        .bind(col)
        .execute(conn.as_mut())
        .await?;

    return Ok(());
}

/// Insert vectorization remarks into the database.
async fn insert_remarks(conn: &mut Transaction<'_, Any>, match_id: i64, vec: i64, width: i64, si: i64) -> Result<(), sqlx::Error> {
    sqlx::query::<Any>("insert into remarks values (?, ?, ?, ?)")
        .bind(match_id)
        .bind(vec)
        .bind(width)
        .bind(si)
        .execute(conn.as_mut())
        .await?;

    return Ok(());
}

/// Search a list of LoopInfo for a loop that matches LINE & COL.
fn find_loop_info(loop_info: &[LoopInfo], line: i64, _col: i64) -> Option<&LoopInfo> {
    for info in loop_info {
        if info.line == line {
            return Some(&info);
        }
    }

    return None;
}

/// Insert the IR mix into the database.
async fn insert_ir_mix(conn: &mut Transaction<'_, Any>, match_id: i64, info: &LoopInfo) -> Result<(), sqlx::Error> {
    sqlx::query::<Any>("insert into ir_mix values (?, ?, ?, ?, ?)")
        .bind(match_id)
        .bind(info.ir_count)
        .bind(info.ir_mem)
        .bind(info.ir_arith)
        .bind(info.ir_other)
        .execute(conn.as_mut())
        .await?;

    return Ok(());
}

/// Insert the IR mix into the database.
async fn insert_mem_pattern(conn: &mut Transaction<'_, Any>, match_id: i64, info: &LoopInfo) -> Result<(), sqlx::Error> {
    sqlx::query::<Any>("insert into pattern values (?, ?, ?)")
        .bind(match_id)
        .bind(info.pat_start)
        .bind(info.pat_step)
        .execute(conn.as_mut())
        .await?;

    return Ok(());
}

/// Insert the IR mix into the database.
async fn insert_si_status(conn: &mut Transaction<'_, Any>, match_id: i64, info: &SIStatus) -> Result<(), sqlx::Error> {
    // FIXME: Hard-coded ids
    let key = match info {
        SIStatus::Enabled       => 0,
        SIStatus::Disabled      => 1,
        SIStatus::FloatingPoint => 2,
        SIStatus::ControlFlow   => 3,
    };

    sqlx::query::<Any>("insert into si_info values (?, ?)")
        .bind(match_id)
        .bind(key)
        .execute(conn.as_mut())
        .await?;

    return Ok(());
}
