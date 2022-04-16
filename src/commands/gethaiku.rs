use crate::{
    database,
    formatting::{format_haiku_embed, to_embed_data},
};
use serenity::{
    async_trait,
    builder::CreateEmbed,
    client::Context,
    model::interactions::{
        application_command::ApplicationCommandInteraction, InteractionResponseType,
    },
};
use slash_helper::{ApplicationCommandInteractionHandler, InvocationError};
use slash_helper_macros::Command;

/// Fetch a specific haiku from this server by its id
#[derive(Command)]
#[name = "gethaiku"]
pub struct GetHaikuCommand {
    /// Id of the haiku to fetch
    id: i64,
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
