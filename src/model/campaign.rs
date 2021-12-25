//! 

use serde::{Serialize, Deserialize};
use std::collections::BTreeMap;

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
);
*/

pub struct Campaign {
    pub guild_id: u64,
    pub campaign_name: String,
    pub xp_thresholds: CampaignLevels,
    pub allow_inventory: bool,
    pub allow_quests: bool,
    pub allow_characters: bool,
    pub monitor_inventory_chanid: Option<u64>,
    pub monitor_quests_chanid: Option<u64>,
    pub monitor_characters_chanid: Option<u64>,  
}

impl Campaign {
    pub fn new(
        guild_id: u64,
        campaign_name: String,
        xp_thresholds: CampaignLevels,
        allow_inventory: bool,
        allow_quests: bool,
        allow_characters: bool,
        monitor_inventory_chanid: Option<u64>,
        monitor_quests_chanid: Option<u64>,
        monitor_characters_chanid: Option<u64>, 
    ) -> Campaign {
        Campaign{guild_id, campaign_name, xp_thresholds, allow_inventory, allow_quests, allow_characters, monitor_inventory_chanid, monitor_quests_chanid, monitor_characters_chanid }
    }
}

#[derive(Serialize, Deserialize)]
pub struct CampaignLevels {
    // HashMap<Level, XPRequired>
    inner: BTreeMap<u64, u64>,
}

impl CampaignLevels {
    pub fn new() -> CampaignLevels {
        let mut table: BTreeMap<u64, u64> = BTreeMap::new();

        // Initialize with 3.5e standard XP by level.
        table.insert(1, 0);
        table.insert(2, 1000);
        table.insert(3, 3000);
        table.insert(4, 6000);
        table.insert(5, 10000);
        table.insert(6, 15000);
        table.insert(7, 21000);
        table.insert(8, 28000);
        table.insert(9, 36000);
        table.insert(10, 45000);
        table.insert(11, 55000);
        table.insert(12, 66000);
        table.insert(13, 78000);
        table.insert(14, 91000);
        table.insert(15, 105000);
        table.insert(16, 120000);
        table.insert(17, 136000);
        table.insert(18, 153000);
        table.insert(19, 171000);
        table.insert(20, 190000);

        CampaignLevels{ inner: table }
    }

    pub fn empty() -> CampaignLevels {
        CampaignLevels{ inner: BTreeMap::new() }
    }

    /// This data structure must adhere to some strict rules.
    /// 
    /// - A level's XP requirement must be greater than every preceding level.
    /// - A level's XP requirement must be less than every following level.
    /// - No two levels must have the same XP requirement.
    pub fn add_level(&mut self, level: u64, xp_required: u64) -> Result<(), String> {
        let is_valid = self.inner.iter().all(|(k, v)| {
            // Check that the value is greater than every preceding level.
            if *k < level {
                return xp_required > *v;
            }
            // Check that the value is less than every following level.
            if *k > level {
                return xp_required < *v;
            }
            // This only hits if *k = level.
            // In this case we are overwriting
            // the given level, so we don't care
            // what it is and give it a blind pass.
            return true;
        });

        if !is_valid {
            // If it's invalid, then construct an
            // error message for the user.
            let left = self.inner.get_key_value(&(level - 1));
            let right = self.inner.get_key_value(&(level + 1));

            let mut sb = String::new();
            if left.is_some() {
                sb.push_str(format!("XP for level {} ({}) must be greater than the XP required for the previous level {} ({}).", 
                    level, 
                    xp_required, 
                    left.unwrap().0, 
                    left.unwrap().1
                ).as_str());
            }

            if left.is_some() && right.is_some() { sb.push_str("\n"); }

            if right.is_some() {
                sb.push_str(format!("XP for level {} ({}) must be less than the XP required for the next level {} ({}).", 
                    level, 
                    xp_required, 
                    right.unwrap().0, 
                    right.unwrap().1
                ).as_str());
            }
            return Err(sb);
        }

        self.inner.insert(level, xp_required);
        Ok(())
    }

    // Fortunately, we have no need to validate anything on data exit.
    pub fn remove_level(&mut self, level: u64) -> Result<(u64, u64), String> {
        match self.inner.remove_entry(&level) {
            Some((k, v)) => Ok((k, v)),
            None => Err(format!("Level {} was not in the table!", level)),
        }
    }

    /// Given a level, retrieve how much XP you need to reach it.
    /// Returns None if the level is not present in the table.
    pub fn xp_from_level(&self, level: u64) -> Option<&u64> {
        self.inner.get(&level)
    }

    /// Given an XP total, retrieves which level it corresponds to.
    /// Returns None if there are no items in the table.
    pub fn level_from_xp(&self, xp: u64) -> Option<u64> {
        if self.inner.is_empty() {
            return None;
        }

        let mut level: u64 = 1;
        for (k, v) in self.inner.iter() {
            // If we have more XP than the current level,
            // we are at LEAST that level.
            if *v <= xp {
                level = *k;
            } // If we have less XP than the threshold then we MUST be the previous level.
            else if *v > xp {
                break
            }
        }
        Some(level)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_campaign_levels_edit_level() {
        let mut levels = CampaignLevels::empty();

        // Assert can add and change valid entries.
        assert_eq!(levels.add_level(1, 0).is_ok(), true);
        assert_eq!(levels.xp_from_level(1), Some(&(0 as u64)));

        assert_eq!(levels.add_level(1, 50).is_ok(), true);
        assert_eq!(levels.xp_from_level(1), Some(&(50 as u64)));

        // Assert does not allow higher level w/ lower XP.
        assert_eq!(levels.add_level(2, 0).is_err(), true);
        assert_eq!(levels.xp_from_level(2), None);
        assert_eq!(levels.add_level(2, 100).is_ok(), true);
        assert_eq!(levels.xp_from_level(2), Some(&(100 as u64)));

        // Assert setting existing entry to invalid state fails.
        assert_eq!(levels.add_level(1, 150).is_err(), true);
        // And old value is unchanged.
        assert_eq!(levels.xp_from_level(1), Some(&(50 as u64)));

        // Assert setting existing entry to new but valid state succeeds.
        assert_eq!(levels.add_level(1, 25).is_ok(), true);
        assert_eq!(levels.xp_from_level(1), Some(&(25 as u64)));

        // Assert deletion works
        assert_eq!(levels.remove_level(2).is_ok(), true);
        assert_eq!(levels.xp_from_level(2), None);
    }

    #[test]
    fn test_xp_conversion() {
        let mut levels = CampaignLevels::new();

        assert_eq!(levels.xp_from_level(1), Some(&(0 as u64)));
        assert_eq!(levels.level_from_xp(0), Some(1 as u64));
        assert_eq!(levels.xp_from_level(2), Some(&(1000 as u64)));
        assert_eq!(levels.level_from_xp(1000), Some(2 as u64));
        assert_eq!(levels.xp_from_level(3), Some(&(3000 as u64)));
        assert_eq!(levels.level_from_xp(3000), Some(3 as u64));
        assert_eq!(levels.xp_from_level(4), Some(&(6000 as u64)));
        assert_eq!(levels.level_from_xp(6000), Some(4 as u64));
        assert_eq!(levels.xp_from_level(5), Some(&(10000 as u64)));
        assert_eq!(levels.level_from_xp(10000), Some(5 as u64));
        assert_eq!(levels.xp_from_level(6), Some(&(15000 as u64)));
        assert_eq!(levels.level_from_xp(15000), Some(6 as u64));
        assert_eq!(levels.xp_from_level(7), Some(&(21000 as u64)));
        assert_eq!(levels.level_from_xp(21000), Some(7 as u64));
        assert_eq!(levels.xp_from_level(8), Some(&(28000 as u64)));
        assert_eq!(levels.level_from_xp(28000), Some(8 as u64));
        assert_eq!(levels.xp_from_level(9), Some(&(36000 as u64)));
        assert_eq!(levels.level_from_xp(36000), Some(9 as u64));
        assert_eq!(levels.xp_from_level(10), Some(&(45000 as u64)));
        assert_eq!(levels.level_from_xp(45000), Some(10 as u64));
        assert_eq!(levels.xp_from_level(11), Some(&(55000 as u64)));
        assert_eq!(levels.level_from_xp(55000), Some(11 as u64));
        assert_eq!(levels.xp_from_level(12), Some(&(66000 as u64)));
        assert_eq!(levels.level_from_xp(66000), Some(12 as u64));
        assert_eq!(levels.xp_from_level(13), Some(&(78000 as u64)));
        assert_eq!(levels.level_from_xp(78000), Some(13 as u64));
        assert_eq!(levels.xp_from_level(14), Some(&(91000 as u64)));
        assert_eq!(levels.level_from_xp(91000), Some(14 as u64));
        assert_eq!(levels.xp_from_level(15), Some(&(105000 as u64)));
        assert_eq!(levels.level_from_xp(105000), Some(15 as u64));
        assert_eq!(levels.xp_from_level(16), Some(&(120000 as u64)));
        assert_eq!(levels.level_from_xp(120000), Some(16 as u64));
        assert_eq!(levels.xp_from_level(17), Some(&(136000 as u64)));
        assert_eq!(levels.level_from_xp(136000), Some(17 as u64));
        assert_eq!(levels.xp_from_level(18), Some(&(153000 as u64)));
        assert_eq!(levels.level_from_xp(153000), Some(18 as u64));
        assert_eq!(levels.xp_from_level(19), Some(&(171000 as u64)));
        assert_eq!(levels.level_from_xp(171000), Some(19 as u64));
        assert_eq!(levels.xp_from_level(20), Some(&(190000 as u64)));
        assert_eq!(levels.level_from_xp(190000), Some(20 as u64));
    }
}