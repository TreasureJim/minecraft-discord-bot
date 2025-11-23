use crate::server_state::ServerState;

// pub async fn listen(state: &ServerState) {
//     let attach = state
//         .docker
//         .attach_container(
//             &state.bot_config.container_name,
//             AttachContainerOptionsBuilder::new()
//                 .stdout(true)
//                 .stream(true),
//         )
//         .await;
//     attach.
// }

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
        assert_eq!(player_joined_catch(&s), Some("sally"));

        // normal with caps
        let s = "[20:41:25 INFO]: Sally joined the game";
        assert_eq!(player_joined_catch(&s), Some("Sally"));

        // Multi word name
        let s = "[20:41:25 INFO]: Sally Whiller joined the game";
        assert_eq!(player_joined_catch(&s), Some("Sally Whiller"));

        // Single character name
        let s = "[20:41:25 INFO]: a joined the game";
        assert_eq!(player_joined_catch(&s), Some("a"));

        // just a space
        let s = "[20:41:25 INFO]:   joined the game";
        assert_eq!(player_joined_catch(&s), Some(" "));
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
