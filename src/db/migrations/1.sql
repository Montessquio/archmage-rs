CREATE TABLE IF NOT EXISTS world (
    id  TEXT PRIMARY KEY,
    val TEXT NULL
);

CREATE TABLE IF NOT EXISTS public.pbp_state (
    guild_id            BIGINT NOT NULL,
    channel_id          BIGINT NOT NULL,
    current_turn_index  BIGINT NOT NULL,
    seconds_remaining   BIGINT NOT NULL,
    flags               BIGINT NOT NULL,
    PRIMARY KEY (guild_id, channel_id)
);


CREATE TABLE IF NOT EXISTS public.pbp_interval (
    guild_id    BIGINT NOT NULL,
    channel_id  BIGINT NOT NULL,
    user_id     BIGINT NOT NULL,
    turn_index  BIGINT NOT NULL,
    flags       BIGINT NOT NULL,
    delay_secs  BIGINT NOT NULL,
    PRIMARY KEY (guild_id, channel_id, user_id, turn_index),
    UNIQUE (guild_id, channel_id, turn_index),
    FOREIGN KEY (guild_id, channel_id) 
        REFERENCES public.pbp_state(guild_id, channel_id)
        ON DELETE CASCADE
);

INSERT INTO world(id, val) 
    VALUES ('schema_version', 1)
    ON CONFLICT DO NOTHING;