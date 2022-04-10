use crate::{
    counting::count_line,
    database,
    formatting::{format_haiku_embed, to_embed_data},
    UptimeStart,
};
use chrono::Utc;
use serenity::{
    async_trait,
    builder::{CreateApplicationCommand, CreateEmbed},
    client::Context,
    model::{
        id::GuildId,
        interactions::{
            application_command::{
                ApplicationCommand, ApplicationCommandInteraction,
                ApplicationCommandInteractionDataOptionValue, ApplicationCommandOptionType,
            },
            InteractionResponseType,
        },
    },
};
use std::env;

pub async fn parse_and_invoke_command(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
) -> Result<(), CommandError> {
    match command.data.name.as_ref() {
        UPTIME_COMMAND_NAME => Ok(UptimeCommand::parse(command)?.invoke(ctx, command).await?),
        COUNT_COMMAND_NAME => Ok(CountCommand::parse(command)?.invoke(ctx, command).await?),
        GET_HAIKU_COMMAND_NAME => Ok(GetHaikuCommand::parse(command)?
            .invoke(ctx, command)
            .await?),
        _ => Err(CommandError::UnknownCommand),
    }
}

pub async fn register_commands(ctx: &Context) -> Result<Vec<ApplicationCommand>, serenity::Error> {
    let guild_id = env::var("TEST_GUILD_ID")
        .expect("Expected a test guild id in the environment")
        .parse()
        .expect("Invalid test guild id id");
    let guild_id = GuildId(guild_id);
    GuildId::set_application_commands(&guild_id, &ctx.http, |commands_builder| {
        commands_builder
            .create_application_command(|command| UptimeCommand::register(command))
            .create_application_command(|command| CountCommand::register(command))
            .create_application_command(|command| GetHaikuCommand::register(command))
    })
    .await
}

#[derive(Debug)]
pub enum ParseError {
    MissingOption,
    InvalidOption,
}

#[derive(Debug)]
pub struct InvocationError;

#[derive(Debug)]
pub enum CommandError {
    Parse(ParseError),
    Invocation(InvocationError),
    UnknownCommand,
}
impl From<ParseError> for CommandError {
    fn from(err: ParseError) -> Self {
        Self::Parse(err)
    }
}

impl From<InvocationError> for CommandError {
    fn from(err: InvocationError) -> Self {
        Self::Invocation(err)
    }
}

#[async_trait]
pub trait Command: Invokable + Sized {
    fn parse(command: &ApplicationCommandInteraction) -> Result<Self, ParseError>;
    fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand;
}

#[async_trait]
pub trait Invokable {
    async fn invoke(
        &self,
        ctx: &Context,
        command: &ApplicationCommandInteraction,
    ) -> Result<(), InvocationError>;
}

pub struct UptimeCommand;
const UPTIME_COMMAND_NAME: &'static str = "uptime";

#[async_trait]
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
impl Invokable for UptimeCommand {
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

pub struct CountCommand {
    phrase: String,
}
const COUNT_COMMAND_NAME: &'static str = "count";

#[async_trait]
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
impl Invokable for CountCommand {
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

pub struct GetHaikuCommand {
    id: i64,
}
const GET_HAIKU_COMMAND_NAME: &'static str = "gethaiku";

#[async_trait]
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
impl Invokable for GetHaikuCommand {
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
