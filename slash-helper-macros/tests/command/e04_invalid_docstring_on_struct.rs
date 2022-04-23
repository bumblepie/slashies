use slash_helper_macros::Command;
use slash_helper::{ApplicationCommandInteractionHandler, InvocationError};
use serenity::{async_trait, client::Context, model::interactions::application_command::ApplicationCommandInteraction};

#[doc]
#[derive(Command)]
#[name = "BadCommand"]
struct BadCommand;

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
