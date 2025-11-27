mod active_features;
mod commands;
#[allow(async_fn_in_trait)]
mod docker;
mod server_state;
mod sql;

use bollard::query_parameters::InspectContainerOptionsBuilder;
use serenity::all::{Command, GuildId, Interaction};
use serenity::async_trait;
use serenity::model::gateway::Ready;
use serenity::prelude::*;
use std::{env, sync::Arc};

use crate::active_features::players::PlayerPresenceLog;
use crate::server_state::{BotConfig, ContextExt, ServerState, ServerStateMutables};

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::Command(command) = interaction {
            log::info!("Received interaction command: {}", command.data.name);
            log::trace!("Received interaction command: {:#?}", command);
            let ctx = commands::Context::new(ctx, command);

            let result = match ctx.command.data.name.as_str() {
                "ping" => commands::ping::run(&ctx).await,
                "restart" => commands::restart::run(&ctx).await,
                "log" => commands::log::run(&ctx).await,
                "snitch_channel_add" => commands::snitch::channel::add::run(&ctx).await,
                "snitch_channel_remove" => commands::snitch::channel::remove::run(&ctx).await,
                "snitch_add" => commands::snitch::user::add::run(&ctx).await,
                "snitch_remove" => commands::snitch::user::remove::run(&ctx).await,
                _ => Ok(()),
            };

            if let Err(why) = result {
                log::error!("Cannot respond to slash command: {why}");
            }
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        log::info!("{} is connected!", ready.user.name);

        let public_commands = vec![
            commands::ping::register(),
            commands::restart::register(),
            commands::log::register(),
            commands::snitch::channel::add::register(),
            commands::snitch::channel::remove::register(),
            commands::snitch::user::add::register(),
            commands::snitch::user::remove::register(),
        ];

        // Guild (Server) specific commands
        if let Some(guild_id) = ctx.get_server_state().await.bot_config.guild_id {
            let guild_id = GuildId::new(guild_id);
            let commands = guild_id
                .set_commands(&ctx.http, public_commands.clone())
                .await;
            log::debug!("I now have the following guild slash commands: {commands:#?}");
        }

        // Works on all servers
        let guild_command = Command::set_global_commands(&ctx.http, public_commands).await;
        log::debug!("I created the following global slash command: {guild_command:#?}");
    }
}

#[tokio::main]
async fn main() {
    env_logger::init_from_env(env_logger::Env::default().filter_or("LOG_LEVEL", "warn"));
    // Its ok if there is no env file to load
    if cfg!(debug_assertions) {
        let _ = dotenvy::dotenv();
    }

    let token = env::var("DISCORD_TOKEN").expect("Expected DISCORD_TOKEN in environment");

    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    let mut client = Client::builder(&token, intents)
        .event_handler(Handler)
        .await
        .expect("Error creating client");

    let server_state = {
        let bot_config = BotConfig::initialise();
        let mutables = ServerStateMutables {
            player_presence_log: PlayerPresenceLog::new()
        };
        let server_state = ServerState {
            docker: bollard::Docker::connect_with_local_defaults()
                .expect("Could not connect to docker"),
            db: sqlx::PgPool::connect(&bot_config.db_addr)
                .await
                .expect("Could not connect to database"),
            bot_config,
            mutables: RwLock::new(mutables),
        };
        Arc::new(server_state)
    };

    let _ = server_state
        .docker
        .inspect_container(
            &server_state.bot_config.container_name,
            Some(InspectContainerOptionsBuilder::new().build()),
        )
        .await
        .unwrap_or_else(|_| {
            panic!(
                "Could not find container: {}",
                server_state.bot_config.container_name
            )
        });

    {
        let mut data = client.data.write().await;
        data.insert::<ServerState>(server_state.clone());
    }

    {
        let http_clone = client.http.clone();
        let server_state_clone = server_state.clone();
        tokio::task::spawn(async move {
            let func = |s: String| {
                let http = http_clone.clone();
                let server_state = server_state.clone();
                async move {
                    // todo: add some sort of graceful shutdown or logging
                    let _ = active_features::players::snitch_player_joined(&server_state, &http, &s).await.map_err(|e| log::error!("Error in snitch_player_joined: {e}"));
                }
            };
            docker::attach_and_listen(&server_state_clone, func).await
        });
    }

    if let Err(why) = client.start().await {
        log::error!("Client error: {:?}", why);
    }
}
