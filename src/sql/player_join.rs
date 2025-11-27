use serenity::all::{ChannelId, GuildId, UserId};
use sqlx::PgPool;

use super::SqlU64;

#[derive(sqlx::FromRow)]
pub struct PlayerJoinServerChannel {
    pub guild_id: SqlU64,
    pub channel_id: SqlU64,
}

impl PlayerJoinServerChannel {
    pub fn new(guild_id: GuildId, channel_id: ChannelId) -> Self {
        Self {
            guild_id: guild_id.get().into(),
            channel_id: channel_id.get().into(),
        }
    }

    pub fn insert_channel(
        &self,
    ) -> sqlx::query::Query<'_, sqlx::Postgres, sqlx::postgres::PgArguments> {
        sqlx::query!(
            "INSERT INTO player_joined_server_channel (guild_id, channel_id) VALUES ($1, $2)",
            self.guild_id.to_db(),
            self.channel_id.to_db()
        )
    }

    pub fn remove_channel(
        &self,
    ) -> sqlx::query::Query<'_, sqlx::Postgres, sqlx::postgres::PgArguments> {
        sqlx::query!(
            "DELETE FROM player_joined_server_channel WHERE guild_id = $1 AND channel_id = $2",
            self.guild_id.to_db(),
            self.channel_id.to_db()
        )
    }
}

#[derive(sqlx::FromRow)]
pub struct PlayerJoinIgnore {
    discord_id: SqlU64,
    player_name: String,
}

impl PlayerJoinIgnore {
    pub fn new(discord_id: UserId, player_name: impl Into<String>) -> Self {
        Self {
            discord_id: discord_id.get().into(),
            player_name: player_name.into(),
        }
    }

    pub fn insert_player(
        &self,
    ) -> sqlx::query::Query<'_, sqlx::Postgres, sqlx::postgres::PgArguments> {
        sqlx::query!(
            "INSERT INTO player_join_ignore (discord_id, player_name) VALUES ($1, $2)",
            self.discord_id.to_db(),
            self.player_name,
        )
    }

    pub fn remove_player(
        &self,
    ) -> sqlx::query::Query<'_, sqlx::Postgres, sqlx::postgres::PgArguments> {
        sqlx::query!(
            "DELETE FROM player_join_ignore WHERE discord_id = $1 AND player_name = $2",
            self.discord_id.to_db(),
            self.player_name,
        )
    }

    pub async fn has_player(pool: &PgPool, name: impl AsRef<str>) -> Result<bool, sqlx::Error> {
        let count = sqlx::query!(
            "SELECT COUNT(player_name) FROM player_join_ignore WHERE player_name LIKE $1",
            name.as_ref()
        )
        .fetch_one(pool)
        .await?
        .count
        .unwrap();

        Ok(count > 0)
    }
}
