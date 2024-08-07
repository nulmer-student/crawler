use super::{
    InitInput, InitResult, CompileInput, CompileResult, Interface,
    InternInput, InternResult, MatchData, PreInput, PreprocessResult
};

use std::{io::{BufRead, Write}, path::PathBuf, process::{Command, Stdio}, str::FromStr};
use std::fs;
use lazy_static::lazy_static;
use log::{error, info};
use regex::Regex;
use sqlx::{self, Row, Transaction};
use sqlx::Any;

/// Communication between the compile & intern phases.
#[derive(Debug)]
struct Match {
    file: PathBuf,
    output: String,
}

lazy_static! {
    /// Parse the vectorization remark output.
    static ref MATCH_PATTERN: Regex = {
        let mut pattern = "".to_string();
        pattern.push_str(r"(\d+):(\d+): remark: vectorized loop \(");
        pattern.push_str(r"vectorization width: (\d+),");
        pattern.push_str(r" interleaved count: (\d+),");
        pattern.push_str(r" scalar interpolation count: (\d+)");
        pattern.push_str(r"\)");
        Regex::new(&pattern).unwrap()
    };

    /// Check if a contains a for loop.
    static ref LOOP_PATTERN: Regex = {
        Regex::new(r"for").unwrap()
    };
}


static PRAGMA: &str = "#pragma clang loop scalar_interpolation(enable)\n";

// =============================================================================
// SI Interface
// =============================================================================

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

    /// Don't use the builtin preprocess method.
    fn preprocess(&self, _input: PreInput) -> PreprocessResult {
        return Ok("".to_string());
    }

    /// Compile a single file using SI cost model.
    fn compile(&self, input: CompileInput) -> CompileResult {
        // Log output
        let mut log = "".to_string();

        // Try to compile the file
        let (succeed, output) = try_compile(&input);
        log.push_str(&output);

        // Return early if the compilation failed
        if !succeed {
            return CompileResult { data: Err(()), to_log: log };
        }

        // If the compilation succeeded, find the matches
        return find_match_data(&input, log);
    }

    fn intern(&self, input: InternInput) -> InternResult {
        // Acquire a database connection
        let mut conn = match input.db.rt.block_on(
            async { input.db.pool.begin().await }
        ){
            Ok(c) => c,
            Err(e) => {
                error!("Failed to acquire connection: {}", e);
                return Err(());
            },
        };

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

                    // Add the file to the files table
                    let file_id = input.db.rt.block_on(ensure_file(
                        &mut conn, &entry.file, input.repo_id
                    ));

                    // Insert the match
                    match file_id {
                        Ok(id) => {
                            let r = input.db.rt.block_on(insert_match(
                                &mut conn, id, &args
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

        input.db.rt.block_on(async {
            match conn.commit().await {
                Ok(_) => {},
                Err(e) => { error!("Failed to commit transaction: {:?}", e) },
            }
        });
        return Ok(());
    }
}

// =============================================================================
// Compile
// =============================================================================

fn get_compile_bin(input: &CompileInput, bin: &str) -> PathBuf {
    let dir = PathBuf::from_str(&input.config.interface.args["llvm"]).unwrap();

    return dir.join(bin);
}

/// Format headers using the -I format.
fn format_headers(headers: &Vec<PathBuf>) -> Vec<String> {
    headers.iter()
           .map(|h| format!("-I{}", h.to_str().unwrap()))
           .collect()
}

/// Return true if the compilation succeeded, & return the output.
fn try_compile(input: &CompileInput) -> (bool, String) {
    // Get the path to clang from the args
    let clang = get_compile_bin(input, "clang");
    let headers = format_headers(input.headers);

    // Run a quick compilation so we can check for errors
    let compile = Command::new("timeout")
        .arg("5")
        .arg(clang)
        .arg("-c")
        .arg(input.file)
        .args(headers)
        .args(["-o", "/dev/null"])
        .output()
        .unwrap();

    // Try to get the output
    let output = match String::from_utf8(compile.stderr) {
        Ok(s) => s,
        Err(e) => {
            error!("Failed to read compilation output for: {:?}", e);
            "".to_string()
        }
    };

    // Return true if the compilation succeeded
    if compile.status.success() {
        return (true, output);
    }

    return (false, output);
}

/// Given a successful header combination, compile the file & find matches.
fn find_match_data(input: &CompileInput, log: String) -> CompileResult {
    // Find the innermost loops in the file
    let loop_lines = find_inner_loops(input);

    // Insert SI pragmas before the inner loops
    let src = insert_pragma(input.file, loop_lines);

    // Compile to find all information
    return find_matches(input, src, log);
}

/// Return a list of line numbers that define innermost loops.
fn find_inner_loops(input: &CompileInput) -> Vec<usize> {
    // Compile the file to LLVM IR
    let clang = get_compile_bin(input, "clang");
    let mut compile = Command::new(clang)
        .stdout(Stdio::piped())
        .arg(input.file)
        .args(["-S", "-emit-llvm", "-g", "-o", "-"])
        .spawn()
        .unwrap();

    // Run the loop finder
    let loop_finder = &input.config.interface.args["loop_finder"];
    let opt = get_compile_bin(input, "opt");
    let find = Command::new(opt)
        .stdin(compile.stdout.take().unwrap())
        .arg(&format!("-load-pass-plugin={}", loop_finder))
        .arg("-passes=print<inner-loop>")
        .args(["-o", "/dev/null"])
        .output()
        .unwrap();

    // Parse the results
    let out = String::from_utf8(find.stderr)
        .expect("Failed to parse loop finder output");
    let lines: Vec<_> = out.lines()
                           .map(|l| l.split(" ").collect::<Vec<_>>()[0])
                           .map(|s| s.parse::<usize>()
                                .expect("Failed to parse integer"))
                           .collect();
    return lines;
}

/// Load a file into a String & insert the SI pragmas.
fn insert_pragma(file: &PathBuf, mut pragma_lines: Vec<usize>) -> String {
    // Load the raw file
    let contents = fs::read_to_string(file).expect("Failed to read file");

    // Ensure the pragma_lines are in sorted order
    pragma_lines.sort();

    // Load the file & insert the pragmas where needed
    let mut acc = "".to_string();
    let mut pragma_i = 0;
    for (i, line) in contents.lines().enumerate() {
        let i = i + 1; // Loop finder uses 1-based indexing

        // Check if this line needs a pragma
        if let Some(pragma_line) = pragma_lines.get(pragma_i) {
            if *pragma_line == i {
                pragma_i += 1;
                if is_for_loop(line) {
                    acc.push_str(PRAGMA);
                }
            }
        }

        // Add the line
        acc.push_str(line);
        acc.push('\n');
    }

    return acc;
}

/// Check if the given string contains a definition for a "for" loop.
fn is_for_loop(str: &str) -> bool {
    return LOOP_PATTERN.is_match(str);
}

/// Find the SI data for a given file.
fn find_matches(input: &CompileInput, src: String, log: String) -> CompileResult {
    return CompileResult { data: Err(()), to_log: log };
}

// =============================================================================
// Intern
// =============================================================================

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

/// Insert a match into the database.
async fn insert_match(pool: &mut Transaction<'_, Any>, file_id: i64, args: &[i64; 5]) -> Result<(), sqlx::Error> {
    sqlx::query::<Any>(
        "insert into matches values (uuid_short(), ?, ?, ?, ?, ?, ?)"
    ).bind(file_id)
     .bind(args[0])
     .bind(args[1])
     .bind(args[2])
     .bind(args[3])
     .bind(args[4])
     .execute(pool.as_mut())
     .await?;

    return Ok(());
}
