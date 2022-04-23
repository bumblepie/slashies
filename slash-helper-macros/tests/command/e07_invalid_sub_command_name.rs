use slash_helper_macros::{Command, SubCommand};
use slash_helper::{ApplicationCommandInteractionHandler, InvocationError};
use serenity::{async_trait, client::Context, model::interactions::application_command::ApplicationCommandInteraction};

/// An command with an invalid subcommand
#[derive(Command)]
#[name = "BadCommand"]
enum BadCommand {
    /// A subcommand with an invalid name
    #[name]
    Sub(SubCommand),
}

#[derive(SubCommand)]
struct SubCommand;

#[async_trait]
impl ApplicationCommandInteractionHandler for BadCommand {
    async fn invoke(
        &self,
        _ctx: &Context,
        _command: &ApplicationCommandInteraction,
    ) -> Result<(), InvocationError> {
        unimplemented!()
    }
}

fn main() {}
