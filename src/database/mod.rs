use tracing::{event, Level};

mod guild_global;
mod campaigns;

pub struct MageDB {
    database: sqlx::SqlitePool
}

impl MageDB {
    pub async fn open() -> MageDB {
        // Initiate a connection to the database file, creating the file if required.
        let database = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(
            sqlx::sqlite::SqliteConnectOptions::new()
                .filename("data/archmage.db")
                .create_if_missing(true),
        )
        .await
        .expect("Couldn't connect to database");

        // Run migrations, which updates the database's schema to the latest version.
        sqlx::migrate!("./migrations").run(&database).await.expect("Couldn't run database migrations");

        MageDB{ database }
    }

    pub async fn get_guild_prefix(&self, id: &u64) -> Option<String> {
        // Try to add a guild to the table of guild settings.
        let strid = id.clone().to_string();
        let query_result =  sqlx::query!(
            "SELECT prefix FROM archmage_guilds WHERE id = ?", 
            strid
        )
        .fetch_one(&self.database)
        .await;
        
        // Destructure the error.
        if let Err(ref e) = query_result {
            if let Some(db_error) = e.as_database_error() {
                if let Some(err_code) = db_error.code() {
                    // Error Code 1555 means that the server is already in the Database.
                    // If it is, we can ignore the error. If not, propagate.
                    if err_code != "0" {
                        event!(Level::ERROR, "Unable to add guild to database (Code {}): {}", err_code, e);
                        return None;
                    }
                }
            }
        }
        Some(query_result.unwrap().prefix)
    }
}