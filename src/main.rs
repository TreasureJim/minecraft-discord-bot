#[allow(async_fn_in_trait)]
mod docker;

use bollard::{Docker, query_parameters::InspectContainerOptionsBuilder};
use serenity::all::EditMessage;
use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;
use std::env;
use std::sync::Arc;

use crate::docker::{get_logs, restart_server};

pub struct GlobalData {
    docker: Docker,
    container_name: String,
}
impl TypeMapKey for GlobalData {
    type Value = Arc<GlobalData>;
}

pub trait ContextExt {
    async fn get_global_data(&self) -> Arc<GlobalData>;
}

impl ContextExt for Context {
    async fn get_global_data(&self) -> Arc<GlobalData> {
        self.data
            .read()
            .await
            .get::<GlobalData>()
            .expect("GlobalData not in TypeMap")
            .clone()
    }
}

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        // Ignore messages from bots to prevent infinite loops
        if msg.author.bot {
            return;
        }

        // Check if the bot is mentioned OR message contains "ping"
        let bot_id = ctx.cache.current_user().id;
        let is_mentioned = msg.mentions_user_id(bot_id);
        let content_lower = msg.content.to_lowercase();

        if content_lower == "ping" {
            if let Err(why) = msg.channel_id.say(&ctx.http, "Pong! 🏓").await {
                println!("Error sending message: {:?}", why);
            }
            return;
        }

        if is_mentioned && content_lower.contains("help") {
            const HELP_STR: &'static str = "help - displays this message.\nping - check if bot is responding.\nrestart - restarts the minecraft server (WARNING - this may lose progress if a backup has not been made).";
            if let Err(why) = msg.channel_id.say(&ctx.http, HELP_STR).await {
                println!("Error sending message: {:?}", why);
            }
            return;
        }

        if is_mentioned && content_lower.contains("restart") {
            let mut restart_msg = match msg
                .channel_id
                .say(&ctx.http, ":arrows_clockwise: Restarting Server..")
                .await
            {
                Ok(message) => message,
                Err(why) => {
                    println!("Error sending message: {:?}", why);
                    return;
                }
            };

            let msg = if let Err(e) = restart_server(&*ctx.get_global_data().await).await {
                format!("Failed to restart:\n{e}")
            } else {
                ":white_check_mark: Server restarted!".to_string()
            };

            let builder = EditMessage::new().content(msg);
            if let Err(why) = restart_msg.edit(&ctx, builder).await {
                println!("Error sending message: {:?}", why);
            };

            return;
        }

        if is_mentioned && content_lower.contains("log") {
            let (logs, log_errors) = get_logs(&*ctx.get_global_data().await).await;

            let mut response = format!(
                ":scroll: Retrieved server logs...\n```{}```",
                logs.join("\n")
            );
            if !log_errors.is_empty() {
                response.push_str("\n\n:x: Some errors were generated while retrieving logs.");
            }

            if let Err(why) = msg.channel_id.say(&ctx.http, response).await {
                println!("Error sending message: {:?}", why);
            }

            return;
        }
    }

    async fn ready(&self, _ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

#[tokio::main]
async fn main() {
    env_logger::init_from_env(env_logger::Env::default().filter_or("MINECRAFT_BOT", "warn"));
    let token = env::var("DISCORD_TOKEN").expect("Expected DISCORD_TOKEN in environment");

    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    let mut client = Client::builder(&token, intents)
        .event_handler(Handler)
        .await
        .expect("Error creating client");

    let global_data = GlobalData {
        docker: bollard::Docker::connect_with_local_defaults()
            .expect("Could not connect to docker"),
        container_name: env::var("CONTAINER_NAME").expect("Expected CONTAINER_NAME in environment"),
    };

    let _ = global_data
        .docker
        .inspect_container(
            &global_data.container_name,
            Some(InspectContainerOptionsBuilder::new().build()),
        )
        .await
        .expect(&format!(
            "Could not find container: {}",
            global_data.container_name
        ));

    {
        let mut data = client.data.write().await;
        data.insert::<GlobalData>(Arc::new(global_data));
    }

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}
