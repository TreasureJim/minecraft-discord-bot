use serenity::all::CreateCommand;

use crate::docker::get_logs;

use super::{CommandResult, Context};

pub async fn run(ctx: &Context) -> CommandResult {
    let (logs, log_errors) = get_logs(ctx.get_server_state().await.as_ref()).await;

    let mut response = format!(
        ":scroll: Retrieved server logs...\n```{}```",
        logs.join("\n")
    );
    if !log_errors.is_empty() {
        response.push_str("\n\n:x: Some errors were generated while retrieving logs.");
    }

    ctx.say(response).await?;

    Ok(())
}

pub fn register() -> CreateCommand {
    CreateCommand::new("log").description("Retrieve the log of the server")
}
