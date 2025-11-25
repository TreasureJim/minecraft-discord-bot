-- Add migration script here
CREATE DOMAIN u64 AS BIGINT;

create table if not EXISTS player_joined_server_channel (
  guild_id u64,
  channel_id u64,
  PRIMARY KEY (guild_id, channel_id)
);
