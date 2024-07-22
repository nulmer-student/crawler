use crate::config::Config;

use tokio::runtime::Runtime;
use sqlx::pool::Pool;
use sqlx::Any;
use sqlx::any::AnyPoolOptions;

pub struct Database {
    pub rt: Runtime,
    pub pool: Pool<Any>,
}

impl Database {
    /// Create a new database using CONFIG.
    pub fn new(config: &Config) -> Self {
        // Required before any sqlx queries
        sqlx::any::install_default_drivers();

        // Initialize runtime & pool
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let pool = rt.block_on(Self::get_pool(config));

        // Initialize db tables
        let db = Self { rt, pool };
        db.init_db();

        return db;
    }

    /// Connect to the database.
    async fn get_pool(config: &Config) -> Pool<Any> {
        AnyPoolOptions::new()
            .connect(&format!(
                "mysql://{}:{}@{}/{}",
                config.database.user,
                config.database.password,
                config.database.host,
                config.database.database,
            ))
            .await
            .expect("failed to connect to db")
    }

    /// Initialize the contents of the database.
    fn init_db(&self) {
        self.rt.block_on(self.create_tables())
               .expect("failed to initialize db");
    }

    /// Create the tables in the database.
    async fn create_tables(&self) -> Result<(), sqlx::Error> {
        sqlx::query(
            "create table if not exists repos (
             repo_id     int,
             name        text,
             clone_url   text,
             stars       int,
             primary key (repo_id)
        )"
        ).execute(&self.pool).await?;

        sqlx::query(
            "create table if not exists mined (
             repo_id     int,
             n_success   int,
             n_error     int,
             time        float,
             primary key (repo_id),
             foreign key (repo_id) references repos
        )"
        ).execute(&self.pool).await?;

        sqlx::query(
            "create table if not exists files (
             file_id     int,
             repo_id     int,
             path        text,
             primary key (file_id),
             foreign key (repo_id) references repos
        )"
        ).execute(&self.pool).await?;

        return Ok(());
    }
}
