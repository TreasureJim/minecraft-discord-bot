use crate::commands::CommandResult;
use std::{
    collections::HashMap,
    sync::{
        Arc,
        atomic::{AtomicU8, Ordering},
    },
    time::{Duration, SystemTime},
};

use serenity::all::{CreateMessage, Http};

use crate::{
    server_state::ServerState,
    sql::{self, player_join::PlayerJoinServerChannel},
};

#[derive(Debug)]
pub struct PlayerPresenceLog(HashMap<String, SystemTime>);

impl PlayerPresenceLog {
    pub fn new() -> Self {
        Self(HashMap::new())
    }
}

impl PlayerPresenceLog {
    pub fn cleanup(&mut self) {
        self.0
            .retain(|_, death_time| PlayerPresenceLog::is_past(death_time));
    }

    pub fn new_player_now(&mut self, player_name: String, live_time: Duration) {
        self.new_player(player_name, SystemTime::now(), live_time);
    }

    pub fn new_player(&mut self, player_name: String, join_time: SystemTime, live_time: Duration) {
        const TIME_TO_CLEANUP: u8 = 50;
        static NEW_PLAYER_COUNT: AtomicU8 = AtomicU8::new(0);
        if NEW_PLAYER_COUNT.fetch_add(1, Ordering::Relaxed) >= TIME_TO_CLEANUP {
            self.cleanup();
            NEW_PLAYER_COUNT.store(0, Ordering::Relaxed);
        }

        if !self.is_record_expired(&player_name) {
            return;
        }
        self.0.insert(player_name, join_time + live_time);
    }

    pub fn is_past(death_time: &SystemTime) -> bool {
        SystemTime::now().duration_since(*death_time).is_ok()
    }

    pub fn is_record_expired(&self, player_name: impl AsRef<str>) -> bool {
        let Some(death_time) = self.0.get(player_name.as_ref()) else {
            return true;
        };
        PlayerPresenceLog::is_past(death_time)
    }
}

pub async fn snitch_player_joined(
    server_state: &Arc<ServerState>,
    http: &Arc<Http>,
    message: &str,
) -> CommandResult {
    const PLAYER_ANNOUNCE_COOLDOWN: Duration = Duration::from_mins(10);

    let Some(player_name) = player_joined_catch(message) else {
        return Ok(());
    };

    if !server_state
        .mutables
        .read()
        .await
        .player_presence_log
        .is_record_expired(player_name)
    {
        log::trace!("{player_name} has rejoined before cooldown expiry. Ignoring.");
        return Ok(());
    }
    server_state
        .mutables
        .write()
        .await
        .player_presence_log
        .new_player_now(player_name.to_string(), PLAYER_ANNOUNCE_COOLDOWN);

    match sql::player_join::PlayerJoinIgnore::has_player(&server_state.db, player_name).await {
        Ok(ignore_player) => {
            if ignore_player {
                log::trace!("{player_name} in ignore list. Not sending message.");
                return Ok(());
            }
        }
        Err(e) => {
            log::error!("DB Error: {e}");
            return Ok(());
        }
    }

    for PlayerJoinServerChannel {
        channel_id,
        guild_id: _,
    } in PlayerJoinServerChannel::get_all_channels(&server_state.db)
        .await
        .expect("The db should respond")
    {
        log::debug!(
            "Sending message in channel {} that {player_name} just joined.",
            channel_id.get()
        );
        serenity::all::ChannelId::new(channel_id.get())
            .send_message(
                &http,
                CreateMessage::new().content(format!("{player_name} just joined the server!")),
            )
            .await?;
    }

    Ok(())
}

pub fn player_joined_catch(s: &str) -> Option<&str> {
    // Check if chat message
    if s.contains("]: <") {
        return None;
    }

    // Returns if suffix doesnt exist
    let s = s.strip_suffix(" joined the game")?;
    let name_start = s.find("]: ")? + 3;
    let name = &s[name_start..];
    Some(name)
}

#[cfg(test)]
mod tests {
    use crate::active_features::players::player_joined_catch;

    #[test]
    fn should_match() {
        // Normal name
        let s = "[20:41:25 INFO]: sally joined the game";
        assert_eq!(player_joined_catch(s), Some("sally"));

        // normal with caps
        let s = "[20:41:25 INFO]: Sally joined the game";
        assert_eq!(player_joined_catch(s), Some("Sally"));

        // Multi word name
        let s = "[20:41:25 INFO]: Sally Whiller joined the game";
        assert_eq!(player_joined_catch(s), Some("Sally Whiller"));

        // Single character name
        let s = "[20:41:25 INFO]: a joined the game";
        assert_eq!(player_joined_catch(s), Some("a"));

        // just a space
        let s = "[20:41:25 INFO]:   joined the game";
        assert_eq!(player_joined_catch(s), Some(" "));
    }

    #[test]
    fn no_match() {
        // chat messages
        assert_eq!(
            player_joined_catch("[13:18:57 INFO]: <BalloonsAndPeople> Wiliam joined the game"),
            None
        );
        assert_eq!(
            player_joined_catch("[13:18:57 INFO]: <John> someone joined the game"),
            None
        );
        assert_eq!(
            player_joined_catch("[13:18:57 INFO]: <Alice> I joined the game yesterday"),
            None
        );
    }
}
