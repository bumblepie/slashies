use std::env;

use crate::UptimeStart;
use chrono::Utc;
use serenity::{
    async_trait,
    builder::CreateApplicationCommand,
    client::Context,
    model::{
        id::GuildId,
        interactions::{
            application_command::{ApplicationCommand, ApplicationCommandInteraction},
            InteractionResponseType,
        },
    },
};

pub type CommandResult = Result<(), ()>;

pub fn parse_command(command: &ApplicationCommandInteraction) -> Result<impl Command, ()> {
    match command.data.name.as_ref() {
        UPTIME_COMMAND_NAME => UptimeCommand::parse(command),
        _ => Err(()),
    }
}

pub async fn register_commands(ctx: &Context) -> Result<Vec<ApplicationCommand>, serenity::Error> {
    let guild_id = env::var("TEST_GUILD_ID")
        .expect("Expected a test guild id in the environment")
        .parse()
        .expect("Invalid test guild id id");
    let guild_id = GuildId(guild_id);
    GuildId::set_application_commands(&guild_id, &ctx.http, |commands_builder| {
        commands_builder.create_application_command(|command| UptimeCommand::register(command))
    })
    .await
}

#[async_trait]
pub trait Command: Sized + Invokable {
    fn parse(command: &ApplicationCommandInteraction) -> Result<Self, ()>;
    fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand;
}

#[async_trait]
pub trait Invokable: Sized {
    async fn invoke(&self, ctx: &Context, command: &ApplicationCommandInteraction)
        -> CommandResult;
}

pub struct UptimeCommand;
const UPTIME_COMMAND_NAME: &'static str = "uptime";

#[async_trait]
impl Command for UptimeCommand {
    fn parse(_command: &ApplicationCommandInteraction) -> Result<Self, ()> {
        Ok(Self)
    }

    fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
        command
            .name(UPTIME_COMMAND_NAME)
            .description("Show how long since the bot was last restarted")
    }
}

#[async_trait]
impl Invokable for UptimeCommand {
    /// Show how long since the bot was last restarted
    async fn invoke(
        &self,
        ctx: &Context,
        command: &ApplicationCommandInteraction,
    ) -> CommandResult {
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
