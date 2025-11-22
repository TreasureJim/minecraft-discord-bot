use serenity::all::CreateCommand;

use super::CommandResult;
use super::Context;

pub async fn run(ctx: &Context) -> CommandResult {
    ctx.say("Pong! :ping_pong:".to_string()).await
}

pub fn register() -> CreateCommand {
    CreateCommand::new("ping").description("A ping command (check if the bot is alive)")
}
