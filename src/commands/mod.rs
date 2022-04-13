use self::{
    count::{CountCommand, COUNT_COMMAND_NAME},
    gethaiku::{GetHaikuCommand, GET_HAIKU_COMMAND_NAME},
    random::{RandomHaikuCommand, RANDOM_HAIKU_COMMAND_NAME},
    search::{SearchCommand, SEARCH_COMMAND_NAME},
    test::{TestCommand, TEST_COMMAND_NAME},
    uptime::{UptimeCommand, UPTIME_COMMAND_NAME},
};
use serenity::{
    client::Context, model::interactions::application_command::ApplicationCommandInteraction,
};
use slash_helper::{ApplicationCommandInteractionHandler, Command, InvocationError, ParseError};

pub mod count;
pub mod gethaiku;
pub mod random;
pub mod search;
pub mod test;
pub mod uptime;

pub enum Commands {
    Uptime(UptimeCommand),
    Count(CountCommand),
    GetHaiku(GetHaikuCommand),
    RandomHaiku(RandomHaikuCommand),
    Search(SearchCommand),
    Test(TestCommand),
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
            TEST_COMMAND_NAME => Ok(Self::Test(TestCommand::parse(command)?)),
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
            Self::Test(command) => command.invoke(ctx, command_interaction).await,
        }
    }
}
