use crawler::interface::{
    InitInput, InitResult, CompileInput, CompileResult, Interface,
    InternInput, InternResult, MatchData, PreInput, PreprocessResult
};

use std::collections::HashMap;
use std::io::{Error, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::str::FromStr;
use std::fs;
use lazy_static::lazy_static;
use log::{error, warn};
use regex::Regex;
use sqlx::{self, Row, Transaction};
use sqlx::Any;

/// Communication between the compile & intern phases.
#[derive(Debug)]
struct Match {
    file: PathBuf,
    output: String,
}

/// Values returned from the loop information pass
#[derive(Debug)]
#[allow(dead_code)]
struct LoopInfo {
    line: i64,
    col: i64,
    ir_count: i64,
    ir_mem: i64,
    ir_arith: i64,
    ir_other: i64,
    pat_start: Option<i64>,
    pat_step: Option<i64>,
}

/// SI status.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
enum SIStatus {
    FloatingPoint,  // Not allowed because of FP instructions
    ControlFlow,    // Not allowed becuase of control flow
    Enabled,        // SI is enabled & allowed. SI will be non zero if cost-effective
    Disabled,       // Not enabled for this loop
}

/// Parsed data from the `-debug-only` output.
type DebugInfo = HashMap<(i64, i64), SIStatus>;

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

    static ref INFO_PATTERN: Regex = {
        let mut pattern = "".to_string();
        pattern.push_str(r"loop info: \(");
        pattern.push_str(r"line: (\d+), ");
        pattern.push_str(r"col: (\d+), ");
        pattern.push_str(r"ir_count: (\d+), ");
        pattern.push_str(r"ir_mem: (\d+), ");
        pattern.push_str(r"ir_arith: (\d+), ");
        pattern.push_str(r"ir_other: (\d+), ");
        pattern.push_str(r"pat_start: (\S+), ");
        pattern.push_str(r"pat_step: (\S+)");
        pattern.push_str(r"\)");
        Regex::new(&pattern).unwrap()
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
                 primary key (match_id),
                 foreign key (file_id) references files)")
                .execute(&input.db.pool).await?;

            let _ = sqlx::query(
                "create table if not exists remarks (
                 match_id    bigint,
                 vector      int,
                 width       int,
                 si          int,
                 primary key (match_id),
                 foreign key (match_id) references matches)")
                .execute(&input.db.pool).await?;

            let _ = sqlx::query(
                "create table if not exists ir_mix (
                 match_id    bigint,
                 count       int,
                 mem         int,
                 arith       int,
                 other       int,
                 primary key (match_id),
                 foreign key (match_id) references matches)")
                .execute(&input.db.pool).await?;

            let _ = sqlx::query(
                "create table if not exists pattern (
                 match_id    bigint,
                 start       int,
                 stride      int,
                 primary key (match_id),
                 foreign key (match_id) references matches)")
                .execute(&input.db.pool).await?;

            let _ = sqlx::query(
                "create table if not exists si_info_types (
                 type_id     int,
                 name        text,
                 primary key (type_id))")
                .execute(&input.db.pool).await?;
            let _ = sqlx::query(
                "insert ignore into si_info_types values
                 (0, 'Enabled'),
                 (1, 'Disabled'),
                 (2, 'Floating Point'),
                 (3, 'Control Flow')")
                .execute(&input.db.pool).await?;

            let _ = sqlx::query(
                "create table if not exists si_info (
                 match_id    bigint,
                 type_id     int,
                 primary key (match_id),
                 foreign key (match_id) references matches,
                 foreign key (type_id) references si_info_types)")
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

        // Try to compile the file & return if it fails. Otherwise, find the
        // match data.
        match try_compile(&input, &mut log) {
            Err(_) => CompileResult { data: Err(()), to_log: log },
            Ok(src) => find_match_data(&input, &mut log, &src),
        }
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

        // Intern the matches
        let result = intern_matches(&mut conn, input.clone());

        // Commit the transaction
        input.db.rt.block_on(async {
            match conn.commit().await {
                Ok(_) => {},
                Err(e) => { error!("Failed to commit transaction: {:?}", e) },
            }
        });

        return result;
    }
}

// =============================================================================
// Compile
// =============================================================================

/// Get the path of a binary in the provied LLVM directory.
fn get_compile_bin(bin: &str) -> PathBuf {
    let dir = PathBuf::from_str(env!("CRAWLER_SI_LLVM")).unwrap();
    return dir.join(bin);
}

/// Format headers using the -I format.
fn format_headers(headers: &Vec<PathBuf>) -> Vec<String> {
    headers.iter()
           .map(|h| format!("-I{}", h.to_str().unwrap()))
           .collect()
}

/// Return true if the compilation succeeded, & return the output.
fn try_compile(input: &CompileInput, log: &mut String) -> Result<Vec<u8>, ()> {
    // Get the path to clang from the args
    let clang = get_compile_bin("clang");
    let headers = format_headers(input.headers);

    // Run a quick compilation so we can check for errors
    let compile = Command::new("timeout")
        .arg("5")
        .arg(clang)
        .arg("-c")
        .arg(input.file)
        .args(headers)
        .args(["-emit-llvm", "-g", "-o", "-",])
        .output()
        .unwrap();

    // Log the command used
    log.push_str("\n==============================\n");
    log.push_str(
        &format!("Try for file {:?}:\nHeaders: {:?}", input.file, input.headers)
    );

    // Try to get the output
    let output = match String::from_utf8(compile.stderr) {
        Ok(s) => s,
        Err(e) => {
            error!("Failed to read compilation output for: {:?}", e);
            "".to_string()
        }
    };
    log.push_str("\nOutput:\n");
    log.push_str("------------------------------\n");
    log.push_str(&output);
    log.push_str("------------------------------\n");

    // If the compilation timed out, print so
    if let Some(code) = compile.status.code() {
        if code == 124 {
            log.push_str("timed out\n");
        }
    }

    // Return true if the compilation succeeded
    let result = match compile.status.success() {
        true => Ok(compile.stdout),
        false => Err(()),
    };

    if result.is_ok() {
        log.push_str("success\n")
    } else {
        log.push_str("failed\n")
    }

    return result;
}

/// Given a successful header combination, compile the file & find matches.
fn find_match_data(input: &CompileInput, log: &mut String, src: &[u8]) -> CompileResult {
    // Find the innermost loops in the file
    let loop_lines = find_inner_loops(input, src);

    // Insert SI pragmas before the inner loops
    let src = match insert_pragma(input.file, loop_lines) {
        Ok(s) => s,
        Err(e) => {
            error!("Failed to insert pragma: {:?}", e);
            return CompileResult { data: Err(()), to_log: log.to_string() };
        },
    };

    // Compile to find all information
    return find_matches(input, src, log);
}

/// Return a list of line numbers that define innermost loops.
fn find_inner_loops(input: &CompileInput, src: &[u8]) -> Vec<usize> {
    // Run the loop finder
    let loop_finder = env!("CRAWLER_SI_LOOPS");
    let opt = get_compile_bin("opt");
    let mut find = Command::new(opt)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .arg(&format!("-load-pass-plugin={}", loop_finder))
        .arg("-passes=print<inner-loop>")
        .args(["-o", "/dev/null"])
        .spawn()
        .unwrap();

    // Send the source file
    let mut stdin = find.stdin.take().unwrap();
    stdin.write_all(src).unwrap();
    drop(stdin);

    // Get the output
    let output = find.wait_with_output().unwrap();

    // Parse the results
    let out = String::from_utf8(output.stderr)
        .expect("Failed to parse loop finder output");
    let lines: Vec<_> = out.lines()
                           .map(|l| l.split(" ").collect::<Vec<_>>()[0])
                           .filter_map(|s| s.parse::<usize>().ok())
                           .collect();
    return lines;
}

/// Load a file into a String & insert the SI pragmas.
fn insert_pragma(file: &PathBuf, mut pragma_lines: Vec<usize>) -> Result<String, Error> {
    // Load the raw file
    let contents = fs::read_to_string(file)?;

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

    return Ok(acc);
}

/// Check if the given string contains a definition for a "for" loop.
fn is_for_loop(str: &str) -> bool {
    return LOOP_PATTERN.is_match(str);
}

/// Find the SI data for a given file.
fn find_matches(input: &CompileInput, src: String, log: &mut String) -> CompileResult {
    let mut compile = Command::new("timeout")
        .arg("10")
        .arg(get_compile_bin("clang"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .args(["-c", "-x", "c"])
        .args(format_headers(input.headers))
        .args(["-o", "-"])
        .args(["-emit-llvm", "-O3", "-Rpass=loop-vectorize"])
        .args(["-mllvm", "-debug-only=loop-vectorize"])
        .arg("-")
        .spawn()
        .unwrap();

    // Log the command used
    log.push_str("\n==============================\n");
    log.push_str(
        &format!("Finding info for file {:?}:\nHeaders: {:?}",
                 input.file, input.headers)
    );

    // Send the source file
    let mut stdin = compile.stdin.take().unwrap();
    stdin.write_all(src.as_bytes()).unwrap();
    drop(stdin);

    // Get the compilation output
    let out = compile.wait_with_output().unwrap();

    // Get the output as a string
    let output = match String::from_utf8(out.stderr) {
        Ok(s) => s,
        Err(e) => {
            error!("Failed to read match data: {}", e);
            return CompileResult { data: Err(()), to_log: log.to_string() };
        },
    };
    log.push_str("\nOutput:\n");
    log.push_str("------------------------------\n");
    log.push_str(&output);
    log.push_str("------------------------------\n");

    // If the compilation was successful, return the stderr
    if out.status.success() {
        // Run the loop info pass
        let info = match loop_info(input, &out.stdout, log) {
            Ok(s) => s,
            Err(e) => {
                error!("Failed to find loop info: {:?}", e);
                "".to_string()
            }
        };
        log.push_str(&info);
        log.push_str("------------------------------\n");

        // Return the result
        let result: MatchData = Box::new(Match {
            // Return the relative path
            file: input.file.strip_prefix(input.root).unwrap().to_path_buf(),
            output: output + &info,
        });
        log.push_str("success\n");
        return CompileResult { data: Ok(result), to_log: log.to_string() };
    }

    // If the compilation timed out, print so
    if let Some(code) = out.status.code() {
        if code == 124 {
            log.push_str("timed out\n");
        }
    }

    // Failed, this shouldn't happen since we already tried to compile
    log.push_str("failed\n");
    return CompileResult { data: Err(()), to_log: log.to_string() };
}

/// Find loop information using the "Information" pass
fn loop_info(input: &CompileInput, src: &[u8], _log: &mut String) -> Result<String, ()> {
    // Spawn opt with the information pass
    let info_pass = env!("CRAWLER_SI_INFO");
    let opt = get_compile_bin("opt");
    let mut cmd = Command::new(opt)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .arg(&format!("-load-pass-plugin={}", info_pass))
        .arg("-passes=print<info>")
        .args(["-o", "/dev/null"])
        .spawn()
        .unwrap();

    // Send the optimized code to stdin
    let mut stdin = cmd.stdin.take().unwrap();
    stdin.write_all(src).unwrap();
    drop(stdin);

    // Get the output
    let output = cmd.wait_with_output().unwrap();
    let info = match String::from_utf8(output.stderr) {
        Ok(s) => s,
        Err(e) => {
            error!("Failed to read information output: {:?}", e);
            return Err(());
        },
    };

    return Ok(info);
}

// =============================================================================
// Intern
// =============================================================================

fn intern_matches(conn: &mut Transaction<'_, Any>, input: InternInput) -> InternResult {
    for m in input.data {
        if let Some(entry) = m.downcast_ref::<Match>() {
            // Parse the loop info
            let loop_info = parse_loop_info(&entry.output);

            // Parse the -debug-only
            let debug_info = parse_vector_debug(&entry.output);

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

/// Return None if input is null, parse otherwise.
fn loop_info_option(input: &str) -> Option<i64> {
    match input {
        "null" => None,
        other => Some(other.parse::<i64>().unwrap()),
    }
}

/// Parse the loop info from INPUT.
fn parse_loop_info(input: &str) -> Vec<LoopInfo> {
    let mut acc = vec![];

    let pattern = &INFO_PATTERN;
    for (_body, [line, col, ir_count, ir_mem, ir_arith, ir_other, pat_start, pat_step])
        in pattern.captures_iter(input).map(|c| c.extract::<8>()) {

        acc.push(LoopInfo {
            line: line.parse::<i64>().unwrap(),
            col: col.parse::<i64>().unwrap(),
            ir_count: ir_count.parse::<i64>().unwrap(),
            ir_mem: ir_mem.parse::<i64>().unwrap(),
            ir_arith: ir_arith.parse::<i64>().unwrap(),
            ir_other: ir_other.parse::<i64>().unwrap(),
            pat_start: loop_info_option(pat_start),
            pat_step: loop_info_option(pat_step),
        });
    }

    return acc;
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

/// Parse the vector debug information.
fn parse_vector_debug(input: &str) -> DebugInfo {
    let mut acc = DebugInfo::new();

    // Find the sections for each loop
    let name_pattern = Regex::new(r"LV: Checking a loop[^:]*:(\d+):(\d+)[^\n]*\n")
        .unwrap();
    let locs: Vec<_> = name_pattern.captures_iter(input).collect();
    let parts: Vec<_> = locs.iter().map(|c| c.get(0).unwrap()).collect();

    // Return if there are no sections
    if locs.len() == 0 {
        return acc;
    }

    // Find the region bounds between sections
    let mut regions: Vec<(usize, usize)> = vec![];
    for i in 1..parts.len()  {
        let start = parts[i - 1].end();
        let end = parts[i].start();
        regions.push((start, end));
    }
    // Add the final region
    regions.push((parts[parts.len() - 1].end(), input.len()));

    // Search for LV(SI) lines in each region
    let fp_pattern = Regex::new(r"LV\(SI\): Not legal to interpolate due to floating point instructions").unwrap();
    let cf_pattern = Regex::new(r"LV\(SI\): Not legal to interpolate due to non-interpolatable recipe").unwrap();
    let en_pattern = Regex::new(r"LV\(SI\): SI enabled").unwrap();

    let mut status: Vec<SIStatus> = vec![];
    for (start, end) in regions.clone() {
        let body = &input[start..end];

        if fp_pattern.is_match(body) {
            status.push(SIStatus::FloatingPoint);
        }

        else if cf_pattern.is_match(body) {
            status.push(SIStatus::ControlFlow);
        }

        else if en_pattern.is_match(body) {
            status.push(SIStatus::Enabled);
        }

        else {
            status.push(SIStatus::Disabled);
        }
    }

    // Match locations to statuses
    assert_eq!(locs.len(), status.len());
    for i in 0..locs.len() {
        let line = locs[i].get(1).unwrap().as_str();
        let line = line.parse().unwrap();
        let col  = locs[i].get(2).unwrap().as_str();
        let col  = col.parse().unwrap();

        acc.insert((line, col), status[i].clone());
    }

    return acc;
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
