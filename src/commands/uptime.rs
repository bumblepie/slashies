use crate::UptimeStart;

use chrono::Utc;
use serenity::{
    async_trait,
    client::Context,
    model::interactions::{
        application_command::ApplicationCommandInteraction, InteractionResponseType,
    },
};
use slash_helper::{ApplicationCommandInteractionHandler, InvocationError};
use slash_helper_macros::Command;

/// Show how long since the bot was last restarted
#[derive(Command)]
#[name = "uptime"]
pub struct UptimeCommand;

#[async_trait]
impl ApplicationCommandInteractionHandler for UptimeCommand {
    async fn invoke(
        &self,
        ctx: &Context,
        command: &ApplicationCommandInteraction,
    ) -> Result<(), InvocationError> {
        let data = ctx.data.read().await;
        let uptime_start_lock = data
            .get::<UptimeStart>()
            .expect("Expected HaikuTracker in TypeMap")
            .clone();
        let uptime = Utc::now().signed_duration_since(uptime_start_lock);
        let days = uptime.num_days();
        let uptime = uptime - chrono::Duration::days(days);
        let hrs = uptime.num_hours();
        let uptime = uptime - chrono::Duration::hours(hrs);
        let mins = uptime.num_minutes();

        command
            .create_interaction_response(&ctx.http, |response| {
                response
                    .kind(InteractionResponseType::ChannelMessageWithSource)
                    .interaction_response_data(|message| {
                        message.content(format!(
                            "Uptime: {} days, {} hours, {} minutes",
                            days, hrs, mins
                        ))
                    })
            })
            .await
            .expect("Could not send uptime message");
        Ok(())
    }
}
