mod commands;
#[allow(async_fn_in_trait)]
mod docker;

use bollard::{Docker, query_parameters::InspectContainerOptionsBuilder};
use serenity::all::{
    Command, CreateInteractionResponse, CreateInteractionResponseMessage, EditMessage, GuildId,
    Interaction,
};
use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;
use std::{env, sync::Arc};

use crate::docker::{get_logs, restart_server};

pub struct GlobalData {
    docker: Docker,
    container_name: String,
}
impl TypeMapKey for GlobalData {
    type Value = Arc<GlobalData>;
}

pub trait ContextExt {
    #[allow(async_fn_in_trait)]
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
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::Command(command) = interaction {
            println!("Received interaction command: {}", command.data.name);
            let ctx = commands::Context::new(ctx, command);

            let result = match command.data.name.as_str() {
                "ping" => commands::ping::run(&ctx),
                "restart" => commands::restart::run(&ctx),
            }
            .await;

            if let Err(why) = result {
                println!("Cannot respond to slash command: {why}");
            }
        }
    }

    async fn message(&self, ctx: Context, msg: Message) {
        // Ignore messages from bots to prevent infinite loops
        if msg.author.bot {
            return;
        }

        // Check if the bot is mentioned OR message contains "ping"
        let bot_id = ctx.cache.current_user().id;
        let is_mentioned = msg.mentions_user_id(bot_id);
        let content_lower = msg.content.to_lowercase();

        // if is_mentioned && content_lower.contains("help") {
        //     const HELP_STR: &'static str = "help - displays this message.\nping - check if bot is responding.\nrestart - restarts the minecraft server (WARNING - this may lose progress if a backup has not been made).";
        //     if let Err(why) = msg.channel_id.say(&ctx.http, HELP_STR).await {
        //         println!("Error sending message: {:?}", why);
        //     }
        //     return;
        // }

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

    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);

        // Guild (Server) specific commands
        // Maybe can use as admin commands on personal testing server
        // let guild_id = GuildId::new(
        //     env::var("GUILD_ID")
        //         .expect("Expected GUILD_ID in environment")
        //         .parse()
        //         .expect("GUILD_ID must be an integer"),
        // );
        //
        // let commands = guild_id
        //     .set_commands(
        //         &ctx.http,
        //         vec![
        //         ],
        //     )
        //     .await;
        // println!("I now have the following guild slash commands: {commands:#?}");

        // Works on all servers
        let guild_command =
            Command::create_global_command(&ctx.http, commands::ping::register()).await;
        println!("I created the following global slash command: {guild_command:#?}");
    }
}

#[tokio::main]
async fn main() {
    env_logger::init_from_env(env_logger::Env::default().filter_or("MINECRAFT_BOT", "warn"));
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
