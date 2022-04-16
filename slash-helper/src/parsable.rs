use serenity::model::{
    channel::PartialChannel,
    guild::{PartialMember, Role},
    interactions::application_command::{
        ApplicationCommandInteractionDataOption, ApplicationCommandInteractionDataOptionValue,
        ApplicationCommandOptionType,
    },
    prelude::User,
};

use crate::ParseError;
pub trait ParsableCommandOption: Sized {
    fn parse_from(
        option: Option<&ApplicationCommandInteractionDataOption>,
    ) -> Result<Self, ParseError>;

    fn application_command_option_type() -> ApplicationCommandOptionType;
    fn is_required() -> bool {
        true
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

impl ParsableCommandOption for bool {
    fn parse_from(
        option: Option<&ApplicationCommandInteractionDataOption>,
    ) -> Result<Self, ParseError> {
        match option
            .ok_or(ParseError::MissingOption)?
            .resolved
            .clone()
            .ok_or(ParseError::MissingOption)?
        {
            ApplicationCommandInteractionDataOptionValue::Boolean(b) => Ok(b),
            _ => Err(ParseError::InvalidOption),
        }
    }

    fn application_command_option_type() -> ApplicationCommandOptionType {
        ApplicationCommandOptionType::Boolean
    }
}

impl ParsableCommandOption for (User, Option<PartialMember>) {
    fn parse_from(
        option: Option<&ApplicationCommandInteractionDataOption>,
    ) -> Result<Self, ParseError> {
        match option
            .ok_or(ParseError::MissingOption)?
            .resolved
            .clone()
            .ok_or(ParseError::MissingOption)?
        {
            ApplicationCommandInteractionDataOptionValue::User(u, pm) => Ok((u, pm)),
            _ => Err(ParseError::InvalidOption),
        }
    }

    fn application_command_option_type() -> ApplicationCommandOptionType {
        ApplicationCommandOptionType::User
    }
}

impl ParsableCommandOption for PartialChannel {
    fn parse_from(
        option: Option<&ApplicationCommandInteractionDataOption>,
    ) -> Result<Self, ParseError> {
        match option
            .ok_or(ParseError::MissingOption)?
            .resolved
            .clone()
            .ok_or(ParseError::MissingOption)?
        {
            ApplicationCommandInteractionDataOptionValue::Channel(c) => Ok(c),
            _ => Err(ParseError::InvalidOption),
        }
    }

    fn application_command_option_type() -> ApplicationCommandOptionType {
        ApplicationCommandOptionType::Channel
    }
}

impl ParsableCommandOption for Role {
    fn parse_from(
        option: Option<&ApplicationCommandInteractionDataOption>,
    ) -> Result<Self, ParseError> {
        match option
            .ok_or(ParseError::MissingOption)?
            .resolved
            .clone()
            .ok_or(ParseError::MissingOption)?
        {
            ApplicationCommandInteractionDataOptionValue::Role(r) => Ok(r),
            _ => Err(ParseError::InvalidOption),
        }
    }

    fn application_command_option_type() -> ApplicationCommandOptionType {
        ApplicationCommandOptionType::Role
    }
}

#[derive(Debug, Clone)]
pub enum Mentionable {
    Role(Role),
    User(User, Option<PartialMember>),
}

impl ParsableCommandOption for Mentionable {
    fn parse_from(
        option: Option<&ApplicationCommandInteractionDataOption>,
    ) -> Result<Self, ParseError> {
        match option
            .ok_or(ParseError::MissingOption)?
            .resolved
            .clone()
            .ok_or(ParseError::MissingOption)?
        {
            ApplicationCommandInteractionDataOptionValue::Role(r) => Ok(Self::Role(r)),
            ApplicationCommandInteractionDataOptionValue::User(u, pm) => Ok(Self::User(u, pm)),
            _ => Err(ParseError::InvalidOption),
        }
    }

    fn application_command_option_type() -> ApplicationCommandOptionType {
        ApplicationCommandOptionType::Mentionable
    }
}

impl ParsableCommandOption for f64 {
    fn parse_from(
        option: Option<&ApplicationCommandInteractionDataOption>,
    ) -> Result<Self, ParseError> {
        match option
            .ok_or(ParseError::MissingOption)?
            .resolved
            .clone()
            .ok_or(ParseError::MissingOption)?
        {
            ApplicationCommandInteractionDataOptionValue::Number(n) => Ok(n),
            _ => Err(ParseError::InvalidOption),
        }
    }

    fn application_command_option_type() -> ApplicationCommandOptionType {
        ApplicationCommandOptionType::Number
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
