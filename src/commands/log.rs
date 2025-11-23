use serenity::all::CreateCommand;

use crate::commands::CommandResult;

pub fn run() -> CommandResult {
    /* if is_mentioned && content_lower.contains("log") {
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
    } */
    todo!()
}

pub fn register() -> CreateCommand {
    CreateCommand::new("log").description("Retrieve the log of the server")
}
