use serenity::all::CreateCommand;

use crate::docker::restart_server;

use super::{CommandResult, Context};

pub async fn run(ctx: &Context) -> CommandResult {
    ctx.say(":arrows_clockwise: Restarting Server..").await?;

    let msg = if let Err(e) = restart_server(ctx.get_server_state().await.as_ref()).await {
        format!("Failed to restart:\n{e}")
    } else {
        ":white_check_mark: Server restarted!".to_string()
    };
    ctx.update_msg(msg).await?;

    Ok(())
}

pub fn register() -> CreateCommand {
    CreateCommand::new("restart").description("Restarts the minecraft server!")
}
