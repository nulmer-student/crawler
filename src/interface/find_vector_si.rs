use super::{
    InitInput, InitResult, CompileInput, CompileResult, Interface, InternInput,
    InternResult, MatchData
};

use std::{io::Write, path::PathBuf, process::{Command, Stdio}};
use lazy_static::lazy_static;
use log::error;
use regex::Regex;
use sqlx::{self, Pool, Row};
use sqlx::Any;

/// Communication between the compile & intern phases.
#[derive(Debug)]
struct Match {
    file: PathBuf,
    output: String,
}

lazy_static! {
    static ref MATCH_PATTERN: Regex = {
        let mut pattern = "".to_string();
        pattern.push_str(r"(\d+):(\d+): remark: vectorized loop \(");
        pattern.push_str(r"vectorization width: (\d+),");
        pattern.push_str(r" interleaved count: (\d+),");
        pattern.push_str(r" scalar interpolation count: (\d+)");
        pattern.push_str(r"\)");
        Regex::new(&pattern).unwrap()
    };
}

pub struct FindVectorSI {}

impl Interface for FindVectorSI {
    /// Create new tables to store the files & matches.
    fn init(&self, input: InitInput) -> InitResult {
        let result: Result<(), sqlx::Error> = input.db.rt.block_on(async {
            let _ = sqlx::query(
                "create table if not exists files (
                 file_id     bigint,
                 repo_id     int,
                 path        text,
                 primary key (file_id),
                 foreign key (repo_id) references repos)")
                .execute(&input.db.pool).await?;

            let _ = sqlx::query(
                "create table if not exists matches (
                 match_id    bigint,
                 file_id     bigint,
                 line        int,
                 col         int,
                 vector      int,
                 width       int,
                 si          int,
                 primary key (match_id),
                 foreign key (file_id) references files)")
                .execute(&input.db.pool).await?;

            return Ok(());
        });

        match result {
            Ok(_) => { return Ok(()); },
            Err(e) => { return Err(e.to_string()); },
        }
    }

    /// Compile a single file using SI cost model.
    fn compile(&self, input: CompileInput) -> CompileResult {
        // Get the path to clang from the args
        let clang = &input.config.interface.args["clang"];

        // Format the headers with "-I"
        let headers: Vec<_> = input
            .headers
            .iter()
            .map(|h| format!("-I{}", h.to_str().unwrap()))
            .collect();

        // Compilation command
        let mut cmd = Command::new(clang);
        let mut compile = cmd
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .arg("-c")
            .args(["-x", "c"])
            .args(headers)
            .args(["-o", "/dev/null"])
            .arg("-emit-llvm")
            .arg("-O3")
            .arg("-Rpass=loop-vectorize")
            .arg("-")
            .spawn()
            .unwrap();

        // Send the input source file
        let mut stdin = compile.stdin.take().unwrap();
        stdin.write_all(input.content.as_bytes()).unwrap();
        drop(stdin);    // Blocks if we don't have this

        // Get the compilation output
        let out = compile.wait_with_output().unwrap();

        // If the compilation was successful, return the stderr
        if out.status.success() {
            // Get the output as a string
            let output = match String::from_utf8(out.stderr) {
                Ok(o) => o,
                Err(e) =>  {
                    error!("Failed to read match data: {}", e);
                    return Err(());
                },
            };

            // Return the result
            let result: MatchData = Box::new(Match {
                file: input.file.strip_prefix(input.root).unwrap().to_path_buf(),
                output,
            });
            return Ok(result);
        }

        // Otherwise, error out
        return Err(());
    }

    fn intern(&self, input: InternInput) -> InternResult {
        for m in input.data {
            if let Some(entry) = m.downcast_ref::<Match>() {
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
                    println!("{:?}", args);

                    // Add the file to the files table
                    let file_id = input.db.rt.block_on(ensure_file(
                        &input.db.pool, &entry.file, input.repo_id
                    ));

                    // Insert the match
                    match file_id {
                        Ok(id) => {
                            let r = input.db.rt.block_on(insert_match(
                                &input.db.pool, id, &args
                            ));

                            match r {
                                Ok(_) => {},
                                Err(e) => error!("Failed to insert match: {}", e),
                            }
                        },
                        Err(e) => error!("Failed to insert file: {}", e),
                    }
                }
            }
        }

        return Ok(());
    }
}

/// Get the file_id of FILE.
async fn file_id(pool: &Pool<Any>, file: &PathBuf, repo: i64) -> Option<i64> {
    let row = sqlx::query::<Any>(
        "select file_id
         from files
         where repo_id = ? and path = ?"
    ).bind(repo)
     .bind(file.to_str())
     .fetch_one(pool)
     .await;

    match row {
        Ok(row) => Some(row.get::<i64, usize>(0)),
        Err(_) => None,
    }
}

async fn ensure_file(pool: &Pool<Any>, file: &PathBuf, repo: i64) -> Result<i64, sqlx::Error> {
    match file_id(pool, file, repo).await {
        Some(id) => Ok(id),
        None => {
            // Insert the file
            let _ = sqlx::query::<Any>(
                "insert into files values (uuid_short(), ?, ?)"
            )
                .bind(repo)
                .bind(file.to_str())
                .execute(pool)
                .await?;

            return Ok(file_id(pool, file, repo).await.unwrap());
        }
    }
}

async fn insert_match(pool: &Pool<Any>, file_id: i64, args: &[i64; 5]) -> Result<(), sqlx::Error> {
    sqlx::query::<Any>(
        "insert into matches values (uuid_short(), ?, ?, ?, ?, ?, ?)"
    ).bind(file_id)
     .bind(args[0])
     .bind(args[1])
     .bind(args[2])
     .bind(args[3])
     .bind(args[4])
     .execute(pool)
     .await?;

    return Ok(());
}
