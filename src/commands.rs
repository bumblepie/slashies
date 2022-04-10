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

pub enum Commands {
    Uptime(UptimeCommand),
    Count(CountCommand),
    GetHaiku(GetHaikuCommand),
    RandomHaiku(RandomHaikuCommand),
}

// To be derived via macro
impl Commands {
    pub fn parse(
        _ctx: &Context,
        command: &ApplicationCommandInteraction,
    ) -> Result<Self, ParseError> {
        match command.data.name.as_ref() {
            UPTIME_COMMAND_NAME => Ok(Self::Uptime(UptimeCommand::parse(command)?)),
            COUNT_COMMAND_NAME => Ok(Self::Count(CountCommand::parse(command)?)),
            GET_HAIKU_COMMAND_NAME => Ok(Self::GetHaiku(GetHaikuCommand::parse(command)?)),
            RANDOM_HAIKU_COMMAND_NAME => Ok(Self::RandomHaiku(RandomHaikuCommand::parse(command)?)),
            _ => Err(ParseError::UnknownCommand),
        }
    }

    pub async fn invoke(
        &self,
        ctx: &Context,
        command_interaction: &ApplicationCommandInteraction,
    ) -> Result<(), InvocationError> {
        match self {
            Self::Uptime(command) => command.invoke(ctx, command_interaction).await,
            Self::Count(command) => command.invoke(ctx, command_interaction).await,
            Self::GetHaiku(command) => command.invoke(ctx, command_interaction).await,
            Self::RandomHaiku(command) => command.invoke(ctx, command_interaction).await,
        }
    }
}

// To be replaced with register_commands!(GuildID?, [CommandType, ...]) macro
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
            .create_application_command(|command| RandomHaikuCommand::register(command))
    })
    .await
}

#[derive(Debug)]
pub enum ParseError {
    MissingOption,
    InvalidOption,
    UnknownCommand,
}

#[derive(Debug)]
pub struct InvocationError;

// To be derivable via macro
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

pub struct RandomHaikuCommand;
const RANDOM_HAIKU_COMMAND_NAME: &'static str = "randomhaiku";

#[async_trait]
impl Command for RandomHaikuCommand {
    fn parse(_command: &ApplicationCommandInteraction) -> Result<Self, ParseError> {
        Ok(Self)
    }

    fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
        command
            .name(RANDOM_HAIKU_COMMAND_NAME)
            .description("Fetch a random haiku from this server")
    }
}

#[async_trait]
impl Invokable for RandomHaikuCommand {
    async fn invoke(
        &self,
        ctx: &Context,
        command: &ApplicationCommandInteraction,
    ) -> Result<(), InvocationError> {
        let haiku_and_id = if let Some(server_id) = command.guild_id {
            let db_connection = database::establish_connection();
            database::get_random_haiku(server_id, &db_connection)
        } else {
            None
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
                            message.add_embed(embed);
                            message
                        })
                })
                .await
                .expect("Failed to send haiku msg");
        }
        Ok(())
    }
}
