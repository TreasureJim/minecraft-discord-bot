use std::sync::Arc;

use serenity::all::{
    CommandInteraction, CreateInteractionResponse, CreateInteractionResponseMessage,
    EditInteractionResponse
};
use thiserror::Error;

use crate::server_state::{ContextExt, ServerState};

pub mod log;
pub mod ping;
pub mod restart;
pub mod snitch;


#[derive(Error, Debug)]
pub enum CommandError {
    #[error("Serenity error: {0}")]
    Serenity(#[from] serenity::Error),
    #[error("Database error: {0}")]
    Db(#[from] sqlx::Error),

    #[error("Guild command triggered not from guild")]
    BadGuildCall,
    #[error("Option was given incorrectly")]
    BadOptionPassed,
    #[error("Invalid option index accessed: {0}")]
    BadOptionIndex(u8),
}
pub type CommandResult = Result<(), CommandError>;

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
            .await?;

        Ok(())
    }

    pub async fn get_server_state(&self) -> Arc<ServerState> {
        self.context.get_server_state().await
    }
}

