-- Add migration script here
create table if not EXISTS player_joined_server_channel (
  guild_id BIGINT NOT NULL,
  channel_id BIGINT NOT NULL,
  PRIMARY KEY (guild_id, channel_id)
);

create table if not EXISTS player_join_ignore (
  discord_id BIGINT NOT NULL PRIMARY KEY,
  player_name VARCHAR(25) NOT NULL
);
