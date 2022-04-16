use crate::counting::count_line;
use serenity::{
    async_trait,
    client::Context,
    model::interactions::{
        application_command::ApplicationCommandInteraction, InteractionResponseType,
    },
};
use slash_helper::{ApplicationCommandInteractionHandler, InvocationError};
use slash_helper_macros::Command;

/// Count the number of syllables in a given phrase
#[derive(Command)]
#[name = "count"]
pub struct CountCommand {
    /// The phrase to count
    phrase: String,
}

#[async_trait]
impl ApplicationCommandInteractionHandler for CountCommand {
    async fn invoke(
        &self,
        ctx: &Context,
        command: &ApplicationCommandInteraction,
    ) -> Result<(), InvocationError> {
        match count_line(&self.phrase) {
            Ok(syllables) => {
                command
                    .create_interaction_response(&ctx.http, |response| {
                        response
                            .kind(InteractionResponseType::ChannelMessageWithSource)
                            .interaction_response_data(|message| {
                                message.content(format!(
                                    "The phrase '{}' has {} syllables",
                                    self.phrase, syllables
                                ))
                            })
                    })
                    .await
                    .expect("Could not send uptime message");
            }
            Err(_) => {
                command
                    .create_interaction_response(&ctx.http, |response| {
                        response
                            .kind(InteractionResponseType::ChannelMessageWithSource)
                            .interaction_response_data(|message| {
                                message.content("Could not count this phrase")
                            })
                    })
                    .await
                    .expect("Could not send uptime message");
            }
        }
        Ok(())
    }
}
