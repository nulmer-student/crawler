use crawler::interface::{
    InitInput, InitResult, CompileInput, CompileResult, Interface,
    InternInput, InternResult, PreInput, PreprocessResult
};
use crate::si::*;

use log::error;

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
