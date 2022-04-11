use super::{ApplicationCommandInteractionHandler, Command, InvocationError, ParseError};
use crate::counting::count_line;
use serenity::{
    async_trait,
    builder::CreateApplicationCommand,
    client::Context,
    model::interactions::{
        application_command::{
            ApplicationCommandInteraction, ApplicationCommandInteractionDataOptionValue,
            ApplicationCommandOptionType,
        },
        InteractionResponseType,
    },
};

pub struct CountCommand {
    phrase: String,
}
pub const COUNT_COMMAND_NAME: &'static str = "count";

impl Command for CountCommand {
    fn parse(command: &ApplicationCommandInteraction) -> Result<Self, ParseError> {
        let phrase = command
            .data
            .options
            .iter()
            .find(|option| option.name == "phrase")
            .ok_or(ParseError::MissingOption)?
            .resolved
            .clone()
            .ok_or(ParseError::MissingOption)?;
        if let ApplicationCommandInteractionDataOptionValue::String(phrase) = phrase {
            Ok(Self { phrase })
        } else {
            Err(ParseError::InvalidOption)
        }
    }

    fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
        command
            .name(COUNT_COMMAND_NAME)
            .description("Count the number of syllables in a given phrase")
            .create_option(|option| {
                option
                    .name("phrase")
                    .description("The phrase to count")
                    .kind(ApplicationCommandOptionType::String)
                    .required(true)
            })
    }
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
