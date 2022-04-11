use crate::UptimeStart;

use super::{ApplicationCommandInteractionHandler, Command, InvocationError, ParseError};
use chrono::Utc;
use serenity::{
    async_trait,
    builder::CreateApplicationCommand,
    client::Context,
    model::interactions::{
        application_command::ApplicationCommandInteraction, InteractionResponseType,
    },
};

pub struct UptimeCommand;
pub const UPTIME_COMMAND_NAME: &'static str = "uptime";

impl Command for UptimeCommand {
    fn parse(_command: &ApplicationCommandInteraction) -> Result<Self, ParseError> {
        Ok(Self)
    }

    fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
        command
            .name(UPTIME_COMMAND_NAME)
            .description("Show how long since the bot was last restarted")
    }
}

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
