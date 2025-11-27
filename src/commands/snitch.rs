use crate::commands::CommandError;
use crate::commands::{CommandResult, Context};
use crate::sql;
use serenity::all::CreateCommand;
use serenity::all::Permissions;
use serenity::all::{CommandOptionType, CreateCommandOption};

pub mod channel {
    use super::*;
    static BASE_COMMAND: &str = "snitch_channel";

    pub mod add {
        use super::*;

        pub async fn run(ctx: &Context) -> CommandResult {
            sql::player_join::PlayerJoinServerChannel::new(
                ctx.command
                    .guild_id
                    .ok_or(crate::commands::CommandError::BadGuildCall)?,
                ctx.command.channel_id,
            )
            .insert_channel()
            .execute(&ctx.get_server_state().await.db)
            .await?;

            ctx.say("This channel will now announce when players join the minecraft server").await?;

            Ok(())
        }

        pub fn register() -> CreateCommand {
            CreateCommand::new(BASE_COMMAND.to_string() + "_add")
                .description("Add channel to announce player joining events")
                .default_member_permissions(Permissions::ADMINISTRATOR)
        }
    }

    pub mod remove {
        use super::*;

        pub async fn run(ctx: &Context) -> CommandResult {
            sql::player_join::PlayerJoinServerChannel::new(
                ctx.command.guild_id.unwrap(),
                ctx.command.channel_id,
            )
            .remove_channel()
            .execute(&ctx.get_server_state().await.db)
            .await?;

            ctx.say("This channel will no longer announce when players join the minecraft server").await?;

            Ok(())
        }

        pub fn register() -> CreateCommand {
            CreateCommand::new(BASE_COMMAND.to_string() + "_remove")
                .description("Remove channel from annouce player joining events")
                .default_member_permissions(Permissions::ADMINISTRATOR)
        }
    }
}

pub mod user {
    use super::*;

    static BASE_COMMAND: &str = "snitch";

    pub mod add {
        use super::*;

        pub async fn run(ctx: &Context) -> CommandResult {
            let options = ctx.command.data.options();
            let player_name = match options.first().ok_or(CommandError::BadOptionIndex(0))?.value {
                serenity::all::ResolvedValue::String(s) => s,
                _ => return Err(CommandError::BadOptionPassed),
            };
            sql::player_join::PlayerJoinIgnore::new(ctx.command.user.id, player_name)
                .insert_player()
                .execute(&ctx.get_server_state().await.db)
                .await?;

            ctx.say(format!("{player_name} will now be announced when they join the minecraft server.")).await?;

            Ok(())
        }

        pub fn register() -> CreateCommand {
            CreateCommand::new(BASE_COMMAND.to_string() + "_add")
                .description("Announces player when they join the minecraft server")
                .add_option(
                    CreateCommandOption::new(
                        CommandOptionType::String,
                        "minecraft_name",
                        "Minecraft player name",
                    )
                    .required(true),
                )
        }
    }

    pub mod remove {
        use super::*;

        pub async fn run(ctx: &Context) -> CommandResult {
            let options = ctx.command.data.options();
            let player_name = match options.first().ok_or(CommandError::BadOptionIndex(0))?.value {
                serenity::all::ResolvedValue::String(s) => s,
                _ => return Err(CommandError::BadOptionPassed),
            };
            sql::player_join::PlayerJoinIgnore::new(ctx.command.user.id, player_name)
                .remove_player()
                .execute(&ctx.get_server_state().await.db)
                .await?;

            ctx.say(format!("{player_name} will no longer be announced when they join the minecraft server.")).await?;

            Ok(())
        }

        pub fn register() -> CreateCommand {
            CreateCommand::new(BASE_COMMAND.to_string() + "_remove")
                .description("Dont announce player when they join the minecraft server")
                .add_option(
                    CreateCommandOption::new(
                        CommandOptionType::String,
                        "minecraft_name",
                        "Minecraft player name",
                    )
                    .required(true),
                )
        }
    }
}
