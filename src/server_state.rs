use std::sync::Arc;

use bollard::Docker;
use serenity::{all::Context, prelude::TypeMapKey};

macro_rules! env_expect {
    ($env_name:literal) => {
        std::env::var($env_name)
            .expect(format!("Expected {} variable in environment", $env_name).as_str())
    };
}

pub struct ServerState {
    pub bot_config: BotConfig,
    pub docker: Docker,
}

impl TypeMapKey for ServerState {
    type Value = Arc<ServerState>;
}

pub trait ContextExt {
    #[allow(async_fn_in_trait)]
    async fn get_server_state(&self) -> Arc<ServerState>;
}

impl ContextExt for Context {
    async fn get_server_state(&self) -> Arc<ServerState> {
        self.data
            .read()
            .await
            .get::<ServerState>()
            .expect("GlobalData not in TypeMap")
            .clone()
    }
}

pub struct BotConfig {
    pub container_name: String,
    pub guild_id: Option<u64>,
}

impl BotConfig {
    pub fn initialise() -> Self {
        Self {
            container_name: env_expect!("CONTAINER_NAME"),
            guild_id: std::env::var("GUILD_ID").ok().map(|id| id.parse().expect("GUILD_ID was not a positive number")),
        }
    }
}
