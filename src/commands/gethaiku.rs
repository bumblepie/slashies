use crate::{
    database,
    formatting::{format_haiku_embed, to_embed_data},
};
use serenity::{
    async_trait,
    builder::{CreateApplicationCommand, CreateEmbed},
    client::Context,
    model::interactions::{
        application_command::{
            ApplicationCommandInteraction, ApplicationCommandInteractionDataOptionValue,
            ApplicationCommandOptionType,
        },
        InteractionResponseType,
    },
};
use slash_helper::{ApplicationCommandInteractionHandler, Command, InvocationError, ParseError};

pub struct GetHaikuCommand {
    id: i64,
}
pub const GET_HAIKU_COMMAND_NAME: &'static str = "gethaiku";

impl Command for GetHaikuCommand {
    fn parse(command: &ApplicationCommandInteraction) -> Result<Self, ParseError> {
        let id = command
            .data
            .options
            .iter()
            .find(|option| option.name == "id")
            .ok_or(ParseError::MissingOption)?
            .resolved
            .clone()
            .ok_or(ParseError::MissingOption)?;
        if let ApplicationCommandInteractionDataOptionValue::Integer(id) = id {
            Ok(Self { id })
        } else {
            Err(ParseError::InvalidOption)
        }
    }

    fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
        command
            .name(GET_HAIKU_COMMAND_NAME)
            .description("Fetch a specific haiku from this server by its id")
            .create_option(|option| {
                option
                    .name("id")
                    .description("Id of the haiku to fetch")
                    .kind(ApplicationCommandOptionType::Integer)
                    .required(true)
            })
    }
}

#[async_trait]
impl ApplicationCommandInteractionHandler for GetHaikuCommand {
    async fn invoke(
        &self,
        ctx: &Context,
        command: &ApplicationCommandInteraction,
    ) -> Result<(), InvocationError> {
        let haiku_and_id = match (self.id, command.guild_id) {
            (id, Some(server_id)) => {
                let db_connection = database::establish_connection();
                database::get_haiku(server_id, id, &db_connection)
            }
            _ => None,
        };
        if let Some((id, haiku)) = haiku_and_id {
            let embed_data = to_embed_data(id, &haiku, ctx).await;
            command
                .create_interaction_response(&ctx.http, |response| {
                    response
                        .kind(InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|message| {
                            let mut embed = CreateEmbed::default();
                            format_haiku_embed(embed_data, &mut embed);
                            message.add_embed(embed)
                        })
                })
                .await
                .expect("Failed to send haiku msg");
        }
        Ok(())
    }
}
