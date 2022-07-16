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

/// This trait contains the functions needed to parse/register a command option
///
/// For an non-required command option, use [`Option<T>`] to make it optional
/// The following types are implemented out of the box:
///
/// | Discord type | Rust type          |
/// |--------------|--------------------|
/// | STRING       | [`String`]         |
/// | INTEGER      | [`i64`]            |
/// | BOOLEAN      | [`bool`]           |
/// | USER         | ([`User`], [`Option<PartialMember>`])|
/// | CHANNEL      | [`PartialChannel`] |
/// | ROLE         | [`Mentionable`]    |
/// | NUMBER       | [`f64`]            |
/// | ATTACHMENT   | N/A                |
pub trait ParsableCommandOption: Sized {
    /// Try to parse this from a command argument provided by an interaction.
    /// The argument might not have been provided, hence the optional input - if this is a non-optional type we would normally
    /// return a [`ParseError::MissingOption`] in this case.
    fn parse_from(
        option: Option<&ApplicationCommandInteractionDataOption>,
    ) -> Result<Self, ParseError>;

    /// The Discord type that this rust type maps to - this will determine how the user fills in the option when using the command in Discord
    fn application_command_option_type() -> ApplicationCommandOptionType;

    /// Whether the option is non-optional. Defaults to `true`.
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

/// An input for the USER Discord type
#[derive(Debug, Clone)]
pub struct UserInput {
    /// The user
    pub user: User,
    /// The user's guild member info (if applicable)
    pub member: Option<PartialMember>,
}

impl ParsableCommandOption for UserInput {
    fn parse_from(
        option: Option<&ApplicationCommandInteractionDataOption>,
    ) -> Result<Self, ParseError> {
        match option
            .ok_or(ParseError::MissingOption)?
            .resolved
            .clone()
            .ok_or(ParseError::MissingOption)?
        {
            ApplicationCommandInteractionDataOptionValue::User(u, pm) => Ok(UserInput {
                user: u,
                member: pm,
            }),
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

/// An input for the MENTIONABLE Discord type
/// Will either be a role or a user
#[derive(Debug, Clone)]
pub enum Mentionable {
    /// A role
    Role(Role),
    /// A user
    User(UserInput),
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
            ApplicationCommandInteractionDataOptionValue::User(u, pm) => {
                Ok(Self::User(UserInput {
                    user: u,
                    member: pm,
                }))
            }
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
