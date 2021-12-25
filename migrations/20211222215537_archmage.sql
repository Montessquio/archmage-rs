-- Add migration script here
-- Contains all data that applies to the guild itself.
CREATE TABLE IF NOT EXISTS archmage_guilds(
    id TEXT PRIMARY KEY, -- The Guild Snowflake
    prefix TEXT NOT NULL -- The discord command prefix to use.
);

-- Contains all campaigns associated with each guild.
CREATE TABLE IF NOT EXISTS archmage_campaigns(
    id INTEGER PRIMARY KEY AUTOINCREMENT, -- No meaning in application logic.
    guild TEXT NOT NULL, -- The Discord Guild Snowflake
    campaign TEXT NOT NULL, -- Campaign ID
    xp BLOB NOT NULL, -- XP autolevelling XP thresholds for the campaign.
    inventory_en BIGINT NOT NULL, -- Feature Gates
    quests_en BIGINT NOT NULL,
    characters_en BIGINT NOT NULL,
    inventory_channel TEXT, -- Monitor Feature Channels. Null if live monitoring is off.
    quest_channel TEXT,
    characters_channel TEXT
);

-- Contains all channels that a campaign is autoassociated with.
CREATE TABLE IF NOT EXISTS archmage_channels(
    id INTEGER PRIMARY KEY AUTOINCREMENT, -- No meaning in application logic.
    guild TEXT NOT NULL, -- The Discord Guild Snowflake
    campaign TEXT NOT NULL, -- Campaign ID within that guild.
    channel TEXT -- Associated channel.
);

-- Contains all OOC user data associated with each campaign.
CREATE TABLE IF NOT EXISTS archmage_users(
    id INTEGER PRIMARY KEY AUTOINCREMENT, -- No meaning in application logic.
    guild TEXT NOT NULL, -- Discord Guild Snowflake.
    user TEXT NOT NULL, -- Discord User Snowflake.
    campaign TEXT NOT NULL, -- Campaign ID must be unique for same guild, but may be the same for different guilds.
    character TEXT, -- Which character does this allow the player to affect? Null if rank is 1 (DM).
    rank BIGINT NOT NULL, -- User permissions for associated Campaign. Currently only 0 (Player) and 1 (DM).
    render BLOB NOT NULL, -- Serialized struct describing how character post should be rendered.
    xp_mul REAL NOT NULL -- XP multiplier for this character.
);

-- Contains Character Inventories.
-- The special character value "pot" contains the party inventory.
-- This means that both !pot and !inv pot work.
CREATE TABLE IF NOT EXISTS archmage_inventory(
    id INTEGER PRIMARY KEY AUTOINCREMENT, -- No meaning in application logic.
    guild TEXT NOT NULL, -- Discord Guild Snowflake.
    user TEXT NOT NULL, -- Discord User Snowflake.
    campaign TEXT NOT NULL, -- Campaign ID must be unique for same guild, but may be the same for different guilds.
    character TEXT NOT NULL, -- Character ID to associate inventory with.
    item TEXT NOT NULL, -- The item name in that character's inventory.
    quantity BIGINT NOT NULL -- How many of the item in that character's inventory.
);

-- Contains Character Bio information, XP, and 
CREATE TABLE IF NOT EXISTS archmage_characters(
    id INTEGER PRIMARY KEY AUTOINCREMENT, -- No meaning in application logic.
    guild TEXT NOT NULL, -- Discord Guild Snowflake.
    user TEXT NOT NULL, -- Discord User Snowflake.
    campaign TEXT NOT NULL, -- Campaign ID must be unique for same guild, but may be the same for different guilds.
    character TEXT NOT NULL, -- Character ID to associate bio with.
    tag	TEXT NOT NULL, -- Character post field title.
    value TEXT NOT NULL -- Character post field body.
);

-- Contains Campaign Quest Tracking.
CREATE TABLE IF NOT EXISTS archmage_quests(
    id INTEGER PRIMARY KEY AUTOINCREMENT, -- No meaning in application logic.
    guild TEXT NOT NULL, -- Discord Guild Snowflake.
    campaign TEXT NOT NULL, -- Campaign ID must be unique for same guild, but may be the same for different guilds.
    name TEXT NOT NULL, -- The quest name.
    description TEXT NOT NULL -- The quests' long-form description.
);