use serenity::{
    async_trait,
    builder::CreateApplicationCommand,
    client::Context,
    model::{
        channel::Message,
        interactions::{
            application_command::{
                ApplicationCommandInteraction, ApplicationCommandInteractionDataOption,
                ApplicationCommandInteractionDataOptionValue, ApplicationCommandOptionType,
            },
            message_component::MessageComponentInteraction,
        },
    },
};

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

pub trait ParsableCommandOption: Sized {
    fn parse_from(
        option: Option<&ApplicationCommandInteractionDataOption>,
    ) -> Result<Self, ParseError>;

    fn application_command_option_type() -> ApplicationCommandOptionType;
    fn is_required() -> bool {
        true
    }
}

impl ParsableCommandOption for i64 {
    fn parse_from(
        option: Option<&ApplicationCommandInteractionDataOption>,
    ) -> Result<Self, ParseError> {
        match option
            .ok_or(ParseError::MissingOption)?
            .resolved
            .clone()
            .ok_or(ParseError::MissingOption)?
        {
            ApplicationCommandInteractionDataOptionValue::Integer(i) => Ok(i),
            _ => Err(ParseError::InvalidOption),
        }
    }

    fn application_command_option_type() -> ApplicationCommandOptionType {
        ApplicationCommandOptionType::Integer
    }
}

impl ParsableCommandOption for String {
    fn parse_from(
        option: Option<&ApplicationCommandInteractionDataOption>,
    ) -> Result<Self, ParseError> {
        match option
            .ok_or(ParseError::MissingOption)?
            .resolved
            .clone()
            .ok_or(ParseError::MissingOption)?
        {
            ApplicationCommandInteractionDataOptionValue::String(s) => Ok(s),
            _ => Err(ParseError::InvalidOption),
        }
    }

    fn application_command_option_type() -> ApplicationCommandOptionType {
        ApplicationCommandOptionType::String
    }
}

impl<T: ParsableCommandOption> ParsableCommandOption for Option<T> {
    fn parse_from(
        option: Option<&ApplicationCommandInteractionDataOption>,
    ) -> Result<Self, ParseError> {
        match option {
            Some(opt) => Ok(Some(T::parse_from(Some(opt))?)),
            None => Ok(None),
        }
    }

    fn application_command_option_type() -> ApplicationCommandOptionType {
        T::application_command_option_type()
    }

    fn is_required() -> bool {
        false
    }
}

#[macro_export]
macro_rules! register_commands {
    ($ctx:expr, $guild_id:expr, [$($cmdType:ty),+]) => {{
        use serenity::model::interactions::application_command::ApplicationCommand;
        use slash_helper::Command;

        if let Some(guild_id) = $guild_id {
            GuildId::set_application_commands(&guild_id, &$ctx.http, |commands_builder| {
                commands_builder
                $(
                    .create_application_command(|command| <$cmdType>::register(command))
                )*
            })
            .await
        } else {
            ApplicationCommand::set_global_application_commands(&$ctx.http, |commands_builder| {
                commands_builder
                $(
                    .create_application_command(|command| <$cmdType>::register(command))
                )*
            })
            .await
        }
    }};
}
