use serenity::all::{
    CommandInteraction, CreateInteractionResponse, CreateInteractionResponseMessage,
    EditInteractionResponse, Interaction,
};

pub mod log;
pub mod ping;
pub mod players;
pub mod restart;

pub type CommandResult = serenity::Result<()>;

pub struct Context {
    pub context: serenity::all::Context,
    pub command: CommandInteraction,
}

impl Context {
    pub fn new(context: serenity::all::Context, command: CommandInteraction) -> Self {
        Self { context, command }
    }

    pub async fn say(&self, str: impl Into<String>) -> CommandResult {
        self.command
            .create_response(
                &self.context.http,
                CreateInteractionResponse::Message(
                    CreateInteractionResponseMessage::new().content(str),
                ),
            )
            .await?;

        Ok(())
    }

    pub async fn update_msg(&self, str: impl Into<String>) -> CommandResult {
        self.command
            .edit_response(
                &self.context.http,
                EditInteractionResponse::new().content(str),
            )
            .await
    }
}
