use serenity::{
    async_trait,
    builder::CreateApplicationCommand,
    client::Context,
    model::{
        channel::Message,
        id::GuildId,
        interactions::{
            application_command::{ApplicationCommand, ApplicationCommandInteraction},
            message_component::MessageComponentInteraction,
        },
    },
};
use std::env;

use self::{
    count::{CountCommand, COUNT_COMMAND_NAME},
    gethaiku::{GetHaikuCommand, GET_HAIKU_COMMAND_NAME},
    random::{RandomHaikuCommand, RANDOM_HAIKU_COMMAND_NAME},
    search::{SearchCommand, SEARCH_COMMAND_NAME},
    uptime::{UptimeCommand, UPTIME_COMMAND_NAME},
};

mod count;
mod gethaiku;
mod random;
mod search;
mod uptime;

pub enum Commands {
    Uptime(UptimeCommand),
    Count(CountCommand),
    GetHaiku(GetHaikuCommand),
    RandomHaiku(RandomHaikuCommand),
    Search(SearchCommand),
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
            SEARCH_COMMAND_NAME => Ok(Self::Search(SearchCommand::parse(command)?)),
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
            Self::Search(command) => command.invoke(ctx, command_interaction).await,
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
            .create_application_command(|command| SearchCommand::register(command))
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
pub trait Command: ApplicationCommandInteractionHandler + Sized {
    fn parse(command: &ApplicationCommandInteraction) -> Result<Self, ParseError>;
    fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand;
}

#[async_trait]
pub trait ApplicationCommandInteractionHandler {
    async fn invoke(
        &self,
        ctx: &Context,
        command: &ApplicationCommandInteraction,
    ) -> Result<(), InvocationError>;
}

#[async_trait]
pub trait MessageComponentInteractionHandler {
    async fn invoke(
        &mut self,
        ctx: &Context,
        interaction: &MessageComponentInteraction,
        original_message: &mut Message,
    );
}
