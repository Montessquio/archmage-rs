//! Manages all Campaign-related commands, such as creating new campaigns, adding DMs to those campaigns, and removing them.
use crate::model::Campaign;
use crate::model::CampaignLevels;
use tracing::{event, Level};

impl super::MageDB {
    /*    
CREATE TABLE IF NOT EXISTS archmage_campaigns(
    id INTEGER PRIMARY KEY AUTOINCREMENT, -- No meaning in application logic.
    guild TEXT NOT NULL, -- The Discord Guild Snowflake
    campaign TEXT NOT NULL, -- Campaign ID
    xp BLOB NOT NULL, -- XP autolevelling XP thresholds for the campaign.
    inventory_en INTEGER NOT NULL, -- Feature Gates
    quests_en INTEGER NOT NULL,
    characters_en INTEGER NOT NULL,
    inventory_channel TEXT, -- Monitor Feature Channels. Null if live monitoring is off.
    quest_channel TEXT,
    characters_channel TEXT
);*/

    // Insert a campaign into the database.
    // Returns Some if the insertion was successful. Returns None if the item already exists. Returns err if there's a real error.
    pub async fn create_campaign(&self, guild_id: &u64, name: &str) -> Result<Option<()>, String>  {
        let id = *guild_id as i64;

        // Check for existing campaign
        if let Ok(r) = self.read_campaign(guild_id, name).await {
            if r.is_some() {
                return Err(format!("A campaign named \"{}\" already exists!", name));
            }
        }

        // Try to insert.
        let bin = bincode::serialize(&CampaignLevels::new()).unwrap();
        let query_result =  sqlx::query!(
            r#"
            INSERT INTO archmage_campaigns (
                guild, campaign, xp, inventory_en, quests_en, characters_en) 
            VALUES (?, ?, ?, 1, 1, 1)"#, 
            id,
            name,
            bin
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
                        event!(Level::ERROR, "Unable to add campaign to database (Code {}): {}", err_code, e);
                        return Err(format!("Unable to add campaign to database (Code {}): {}", err_code, e));
                    } else {
                        return Ok(None);
                    }
                }
            }
        }

        Ok(Some(()))
    }

    /// Remove a campaign from the database.
    pub async fn delete_campaign(&self, guild_id: &u64, name: &str) -> Result<(), String> {
        let id = *guild_id as i64;
        let query_result =  sqlx::query!(
            r#"
            DELETE FROM archmage_campaigns WHERE guild = ? AND campaign = ?"#, 
            id,
            name,
        )
        .execute(&self.database)
        .await;
        
        // Destructure the error.
        if let Err(e) = query_result {
            if let Some(db_error) = e.as_database_error() {
                if let Some(err_code) = db_error.code() {
                    event!(Level::ERROR, "Unable to remove campaign from database (Code {}): {}", err_code, e);
                    return Err(format!("Unable to remove campaign from database (Code {}): {}", err_code, e));
                }
            }
        }

        Ok(())
    }

    /// Retrieve all campaigns associated with a particular guild.
    pub async fn read_campaigns(&self, guild_id: &u64) -> Result<Vec<Campaign>, String> {
        let id = *guild_id as i64;

        let query_results =  sqlx::query!(
            r#"
            SELECT * FROM archmage_campaigns WHERE guild = ?"#, 
            id,
        )
        .fetch_all(&self.database)
        .await;

        if let Err(ref e) = query_results {
            if let Some(db_error) = e.as_database_error() {
                if let Some(err_code) = db_error.code() {
                    event!(Level::ERROR, "Unable to get campaign from database (Code {}): {}", err_code, e);
                    return Err(format!("Unable to get campaign from database (Code {}): {}", err_code, e));
                }
            }
        }

        return Ok(query_results.unwrap().iter().map(|r| {
            Campaign::new(
                r.guild.parse::<u64>().unwrap(), 
                r.campaign.clone(), 
                bincode::deserialize(&r.xp).unwrap(), 
                r.inventory_en != 0, 
                r.quests_en != 0, 
                r.characters_en != 0, 
                r.inventory_channel.clone().unwrap_or_else(|| "a".to_owned()).parse::<u64>().ok(), 
                r.quest_channel.clone().unwrap_or_else(|| "a".to_owned()).parse::<u64>().ok(), 
                r.characters_channel.clone().unwrap_or_else(|| "a".to_owned()).parse::<u64>().ok()
            )
        }).collect());
    }

    /// Retrieve one campaign by name.
    pub async fn read_campaign(&self, guild_id: &u64, campaign_name: &str) -> Result<Option<Campaign>, String> {
        let id = *guild_id as i64;

        let query_result =  sqlx::query!(
            r#"
            SELECT * FROM archmage_campaigns WHERE guild = ? AND campaign = ?"#, 
            id,
            campaign_name
        )
        .fetch_optional(&self.database)
        .await;

        if let Err(ref e) = query_result {
            if let Some(db_error) = e.as_database_error() {
                if let Some(err_code) = db_error.code() {
                    event!(Level::ERROR, "Unable to get campaign from database (Code {}): {}", err_code, e);
                    return Err(format!("Unable to get campaign from database (Code {}): {}", err_code, e));
                }
            }
        }

        if let Some(r) = query_result.unwrap() {
            Ok(Some(Campaign::new(
                r.guild.parse::<u64>().unwrap(), 
                r.campaign.clone(), 
                bincode::deserialize(&r.xp).unwrap(), 
                r.inventory_en != 0, 
                r.quests_en != 0, 
                r.characters_en != 0, 
                r.inventory_channel.clone().unwrap_or_else(|| "a".to_owned()).parse::<u64>().ok(), 
                r.quest_channel.clone().unwrap_or_else(|| "a".to_owned()).parse::<u64>().ok(), 
                r.characters_channel.clone().unwrap_or_else(|| "a".to_owned()).parse::<u64>().ok()
                )
            ))
        } else {
            Ok(None)
        }
    }
}