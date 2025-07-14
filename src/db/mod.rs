use eyre::Result;
use include_dir::{include_dir, Dir};
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use std::num::ParseIntError;
use thiserror::Error;

mod pbp;

const MIGRATIONS: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/src/db/migrations");
const CURRENT_DB_SCHEMA_VERSION: i32 = 1;

#[derive(Error, Debug)]
pub enum DBError {
    #[error("Query Error: {0}")]
    QueryError(#[from] sqlx::Error),

    #[error("Casting Error: {0}")]
    ParseError(#[from] ParseIntError),

    #[error("Migration \"{0}.sql\" not found.")]
    NoMigrationFound(i32),

    #[error("Database schema version is newer than program version.")]
    IncompatibleDatabase,
}

pub struct ArchmageDatabase {
    pool: Pool<Postgres>,
}

impl ArchmageDatabase {
    /// Initialize a new database connection and
    /// verify the database integrity, upgrading
    /// if necessary.
    pub async fn connect() -> Result<Self> {
        let db_url = format!(
            "postgres://{}:{}@{}:{}/{}",
            crate::CONFIG.db_user,
            crate::CONFIG.db_password,
            crate::CONFIG.db_url,
            crate::CONFIG.db_port,
            crate::CONFIG.db_name
        );

        let pool = PgPoolOptions::new()
            .max_connections(crate::CONFIG.db_max_connections) // Adjust based on workload
            .connect(&db_url)
            .await?;

        let s = Self { pool };

        s.init_upgrade().await?;

        Ok(s)
    }

    /// Returns the version of the DB schema, or 0 if the DB is not set up.
    pub async fn version(&self) -> Result<i32, DBError> {
        let (world_exists, ): (bool, ) = sqlx::query_as("SELECT EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'public' AND table_name = 'world')")
            .fetch_one(&self.pool)
            .await?;

        match world_exists {
            false => Ok(0),
            true => sqlx::query_as("SELECT val FROM world WHERE id = 'schema_version'")
                .fetch_optional(&self.pool)
                .await?
                .map(|(s,): (String,)| s.parse::<i32>())
                .transpose()?
                .map_or(Ok(0), Ok),
        }
    }

    /// Apply migrations, initializing the DB if it isn't set up yet.
    async fn init_upgrade(&self) -> Result<(), DBError> {
        let version = self.version().await?;

        if version > CURRENT_DB_SCHEMA_VERSION {
            return Err(DBError::IncompatibleDatabase); // Special case: database is newer
        }

        for mv in (version + 1)..=CURRENT_DB_SCHEMA_VERSION {
            let path = {
                let mut m = mv.to_string();
                m.push_str(".sql");
                m
            };
            let migration = match MIGRATIONS.get_file(path) {
                Some(f) => f.contents_utf8().expect("Migration file was not UTF-8!"),
                None => return Err(DBError::NoMigrationFound(mv)),
            };
            let _ = sqlx::raw_sql(migration).execute(&self.pool).await?;
        }

        Ok(())
    }
}
