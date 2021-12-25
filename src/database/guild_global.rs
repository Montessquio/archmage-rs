use tracing::{event, Level};

impl super::MageDB {
    pub async fn set_up_new_guild(&self, id: &u64) {
        // Try to add a guild to the table of guild settings.
        let strid = id.clone().to_string();
        let query_result =  sqlx::query!(
            "INSERT INTO archmage_guilds (id, prefix) VALUES (?, '!')", 
            strid
        )
        .execute(&self.database)
        .await;
        
        // Destructure the error.
        if let Err(e) = query_result {
            if let Some(db_error) = e.as_database_error() {
                if let Some(err_code) = db_error.code() {
                    // Error Code 1555 means that the server is already in the Database.
                    // If it is, we can ignore the error. If not, propagate.
                    if err_code != "1555" {
                        event!(Level::ERROR, "Unable to add guild to database (Code {}): {}", err_code, e);
                    }
                }
            }
        }
    }

    pub async fn set_prefix(&self, guild_id: &u64, prefix: &str) -> Result<(), String> {
        let strid = guild_id.clone().to_string();
        let query_result =  sqlx::query!(
            "UPDATE archmage_guilds SET prefix = ? WHERE id = ?",
            prefix,
            strid
        )
        .execute(&self.database)
        .await;
        
        // Destructure the error.
        if let Err(e) = query_result {
            if let Some(db_error) = e.as_database_error() {
                if let Some(err_code) = db_error.code() {
                    event!(Level::ERROR, "Unable to change guild prefix (Code {}): {}", err_code, e);
                    return Err(format!("Unable to change guild prefix (Code {}): {}", err_code, e));
                }
            }
        }

        Ok(())
    }
}