use crate::{
    database,
    formatting::{format_haiku_embed, to_embed_data},
    models::Haiku,
    MessageComponentInteractionHandlers,
};
use serenity::{
    async_trait,
    builder::CreateEmbed,
    client::Context,
    model::{
        channel::Message,
        interactions::{
            application_command::ApplicationCommandInteraction,
            message_component::{ButtonStyle, MessageComponentInteraction},
            InteractionResponseType,
        },
    },
};
use slash_helper::{
    ApplicationCommandInteractionHandler, InvocationError, MessageComponentInteractionHandler,
};
use slash_helper_macros::Command;

/// Search for a haiku
#[derive(Command)]
#[name = "search"]
pub struct SearchCommand {
    /// A set of keywords to search for, separated by spaces
    keywords: String,
}

#[async_trait]
impl ApplicationCommandInteractionHandler for SearchCommand {
    async fn invoke(
        &self,
        ctx: &Context,
        command: &ApplicationCommandInteraction,
    ) -> Result<(), InvocationError> {
        let keywords = self
            .keywords
            .split_whitespace()
            .map(|word| word.to_owned())
            .collect::<Vec<String>>();

        if let Some(server_id) = command.guild_id {
            let db_connection = database::establish_connection();
            let search_results = database::search_haikus(server_id, keywords, &db_connection);
            if search_results.is_empty() {
                command
                    .create_interaction_response(&ctx.http, |response| {
                        response
                            .kind(InteractionResponseType::ChannelMessageWithSource)
                            .interaction_response_data(|message| {
                                message.content("No haikus found for search terms.")
                            })
                    })
                    .await
                    .expect("Could not send search results message");
            } else {
                let search_index = 0;
                let (id, haiku) = search_results.get(search_index).unwrap();
                let embed_data = to_embed_data(*id, &haiku, ctx).await;
                command
                    .create_interaction_response(&ctx.http, |response| {
                        response
                            .kind(InteractionResponseType::ChannelMessageWithSource)
                            .interaction_response_data(|message| {
                                let mut embed = CreateEmbed::default();
                                format_haiku_embed(embed_data, &mut embed);
                                message.add_embed(embed);
                                message.content(format!(
                                    "Search result {}/{}",
                                    search_index + 1,
                                    search_results.len()
                                ));
                                message.components(|components| {
                                    components.create_action_row(|row| {
                                        row.create_button(|button| {
                                            button
                                                .custom_id("previous")
                                                .label("Previous")
                                                .style(ButtonStyle::Primary)
                                                .disabled(search_index < 1)
                                        })
                                        .create_button(
                                            |button| {
                                                button
                                                    .custom_id("next")
                                                    .label("Next")
                                                    .style(ButtonStyle::Primary)
                                                    .disabled(
                                                        search_index >= search_results.len() - 1,
                                                    )
                                            },
                                        )
                                    })
                                });
                                message
                            })
                    })
                    .await
                    .expect("Failed to send search results");
                let handler = Box::new(SearchReactionHandler {
                    search_index,
                    search_results,
                });
                let data = ctx.data.read().await;
                let handlers = data
                    .get::<MessageComponentInteractionHandlers>()
                    .expect("Expected Handlers in TypeMap");
                handlers.insert(command.id, handler);
            }
        }
        Ok(())
    }
}

pub struct SearchReactionHandler {
    search_index: usize,
    search_results: Vec<(i64, Haiku)>,
}

#[async_trait]
impl MessageComponentInteractionHandler for SearchReactionHandler {
    async fn invoke(
        &mut self,
        ctx: &Context,
        interaction: &MessageComponentInteraction,
        original_message: &mut Message,
    ) {
        let new_index = match interaction.data.custom_id.as_str() {
            "next" => Some(self.search_index + 1),
            "previous" => self.search_index.checked_sub(1),
            _ => None,
        };
        if let Some((new_index, (id, haiku))) = new_index
            .map(|i| self.search_results.get(i).map(|haiku| (i, haiku)))
            .flatten()
        {
            let embed_data = to_embed_data(*id, &haiku, ctx).await;
            original_message
                .edit(&ctx.http, |message| {
                    message
                        .set_embeds(Vec::new())
                        .add_embed(|embed| format_haiku_embed(embed_data, embed))
                        .content(format!(
                            "Search result {}/{}",
                            new_index + 1,
                            self.search_results.len()
                        ))
                        .components(|components| {
                            components.create_action_row(|row| {
                                row.create_button(|button| {
                                    button
                                        .custom_id("previous")
                                        .label("Previous")
                                        .style(ButtonStyle::Primary)
                                        .disabled(new_index < 1)
                                })
                                .create_button(|button| {
                                    button
                                        .custom_id("next")
                                        .label("Next")
                                        .style(ButtonStyle::Primary)
                                        .disabled(new_index >= self.search_results.len() - 1)
                                })
                            })
                        });
                    message
                })
                .await
                .expect("Failed to send search results");
            self.search_index = new_index;
            interaction
                .create_interaction_response(&ctx.http, |response| {
                    response.kind(InteractionResponseType::UpdateMessage)
                })
                .await
                .expect("Failed to respond to component interaction");
        }
    }
}
