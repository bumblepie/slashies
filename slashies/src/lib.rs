//! A simple way to create slash commands for Discord bots
//!
//! Slashies helps to reduce the boiler plate code needed to create slashcommands for a Discord bot.
//! It is built on top of [`Serenity`]. It focuses on providing traits that you can derive using the
//! macros crate for most straightforward use cases, but gives you the escape hatch of implementing
//! these traits yourself if you want to do something more complex.
//!
//! [`Serenity`]: serenity
//!
//! Make sure to read the [Discord documentation](https://discord.com/developers/docs/interactions/application-commands)
//! on slash commands to understand the general concepts like interactions.
//!
//! With Slashies, you can create a slash command in four easy steps:
//! ```no_run
//! # use slashies::*;
//! # use slashies::parsable::*;
//! # use slashies_macros::*;
//! # use serenity::async_trait;
//! # use serenity::prelude::*;
//! # use serenity::model::prelude::*;
//! # use serenity::model::prelude::application_command::*;
//! // 1. Create a struct representing the arguments for the command and derive/implement the
//! // Command trait
//!
//! /// Greet a user
//! #[derive(Debug, Command)]
//! #[name = "greet"]
//! struct HelloCommand {
//!     /// The user to greet
//!     user: UserInput,
//! }
//!
//! // 2. Implement the ApplicationCommandInteractionHandler trait to define what happens when you
//! // call the command
//! #[async_trait]
//! impl ApplicationCommandInteractionHandler for HelloCommand {
//!    async fn invoke(
//!        &self,
//!        ctx: &Context,
//!        command: &ApplicationCommandInteraction,
//!    ) -> Result<(), InvocationError> {
//!        let nickname = self.user.member.as_ref().map(|pm| pm.nick.as_ref()).flatten();
//!        let greeting = if let Some(nick) = nickname {
//!            format!("Hello {} aka {}", self.user.user.name, nick)
//!        } else {
//!            format!("Hello {}", self.user.user.name)
//!        };
//!        command
//!            .create_interaction_response(&ctx.http, |response| {
//!                response
//!                    .kind(InteractionResponseType::ChannelMessageWithSource)
//!                    .interaction_response_data(|message| message.content(greeting))
//!            })
//!            .await
//!            .map_err(|_| InvocationError)
//!    }
//! }
//!
//! // 3. Add the command to an enum that implements the Commands trait, representing all the
//! // commands for the bot
//! #[derive(Debug, Commands)]
//! enum BotCommands {
//!     Hello(HelloCommand),
//! }
//!
//! // 4. Add the basic code to register the command via a macro and handle interactions
//! struct Handler;
//!
//! #[async_trait]
//! impl EventHandler for Handler {
//!     async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
//!         match interaction {
//!             Interaction::ApplicationCommand(command_interaction) => {
//!                 BotCommands::parse(&ctx, &command_interaction)
//!                     .expect("Failed to parse command")
//!                     .invoke(&ctx, &command_interaction)
//!                     .await
//!                     .expect("Failed to invoke command");
//!             }
//!             _ => (),
//!         }
//!     }
//!
//!     async fn ready(&self, ctx: Context, ready: Ready) {
//!         register_commands!(&ctx, None, [HelloCommand])
//!             .expect("Unable to register commands");
//!     }
//! }
//!
//! #[tokio::main]
//! async fn main() {
//!     let token = std::env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
//!     let application_id = std::env::var("DISCORD_USER_ID")
//!         .expect("Expected a user id in the environment")
//!         .parse::<u64>()
//!         .expect("Invalid user id");
//!     let mut client = Client::builder(&token, GatewayIntents::empty())
//!         .event_handler(Handler)
//!         .application_id(application_id)
//!         .await
//!         .expect("Err creating client");
//!
//!     if let Err(why) = client.start().await {
//!         println!("Client error: {:?}", why);
//!     }
//! }
//! ```

#![warn(missing_docs)]
use serenity::{
    async_trait,
    builder::{CreateApplicationCommand, CreateApplicationCommandOption},
    client::Context,
    model::{
        channel::Message,
        interactions::{
            application_command::{
                ApplicationCommandInteraction, ApplicationCommandInteractionDataOption,
            },
            message_component::MessageComponentInteraction,
        },
    },
};

/// This module contains logic for parsing Discord types from interactions into rust types
pub mod parsable;

/// An error that occured while trying to parse a command
#[derive(Debug, Clone)]
pub enum ParseError {
    /// A required option was missing
    MissingOption,
    /// An option was malformed
    InvalidOption,
    /// The command was not one we know about
    UnknownCommand,
}

/// An error that occured while trying to invoke a command
#[derive(Debug, Clone)]
pub struct InvocationError;

/// This trait provides the methods needed to parse and register a slash command.
///
/// For most use cases, just derive it via the macros crate:
/// ```
/// # use slashies::*;
/// # use slashies::parsable::*;
/// # use slashies_macros::*;
/// # use serenity::async_trait;
/// # use serenity::prelude::*;
/// # use serenity::model::prelude::application_command::*;
/// /// Greet a user
/// #[derive(Debug, Command)]
/// #[name = "greet"]
/// struct HelloCommand {
///     /// The user to greet
///     user: UserInput,
/// }
/// # #[async_trait]
/// # impl ApplicationCommandInteractionHandler for HelloCommand {
/// #    async fn invoke(
/// #        &self,
/// #        ctx: &Context,
/// #        command: &ApplicationCommandInteraction,
/// #    ) -> Result<(), InvocationError> {
/// #        unimplemented!()
/// #    }
/// # }
/// ```
/// To derive the trait, you must provide the following (see the example above):
/// - Docstrings for the struct and all fields (these will be used for the
/// descriptions of the command and its options)
/// - The name of the command via the `name` attribute
///
/// All fields must implement the [`parsable::ParsableCommandOption`] trait - see the docs for the
/// trait for a list of types supported out of the box.
///
/// You may also provide additional attributes to specify more complex behaviours for the command
/// options:
///
/// | Attribute     | Explanation                                                                                                         | Examples                                                 | Applicable Discord types |
/// |---------------|---------------------------------------------------------------------------------------------------------------------|----------------------------------------------------------|--------------------------|
/// | choice        | Limits the user's input to specific choices - use the attribute on the field multiple times, once for each choice.  | `#[choice("Action")]` `#[choice("First", 1)]`            | STRING, INTEGER, NUMBER  |
/// | min           | Limits the user's input to be at least this value.                                                                  | `#[min = 0.0]`                                           | INTEGER, NUMBER          |
/// | max           | Limits the user's input to be at most this value.                                                                   | `#[max = 10.0]`                                          | INTEGER, NUMBER          |
/// | channel_types | Limits the user's choice of channels to specific types of channels                                                  | `#[channel_types(ChannelType::Text, ChannelType::News)]` | CHANNEL                  |
///
/// For how to work with subcommands, see the documentation for the [`SubCommand`] trait
pub trait Command: ApplicationCommandInteractionHandler + Sized {
    /// Try to parse the interaction as this type of command
    fn parse(command: &ApplicationCommandInteraction) -> Result<Self, ParseError>;
    /// Register this command so that it can be used
    fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand;
    /// The name of the command
    fn name() -> String;
}

/// This trait provides the functions necessary to parse and register a subcommand for a slash
/// command.
///
/// For most use cases:
/// ```
/// # use serenity::async_trait;
/// # use serenity::prelude::*;
/// # use serenity::model::interactions::*;
/// # use serenity::model::prelude::application_command::*;
/// # use slashies::*;
/// # use slashies_macros::*;
/// # use slashies::parsable::*;
/// // 1. Create the subcommand in the same way you would create a Command, but derive the
/// // SubCommand trait instead
/// // (Remember to implement ApplicationCommandInteractionHandler)
///
/// #[derive(Debug, SubCommand)]
/// struct TestSubCommandOne {
///     /// A number
///     number: f64,
/// }
/// # #[async_trait]
/// # impl ApplicationCommandInteractionHandler for TestSubCommandOne {
/// #     async fn invoke(
/// #         &self,
/// #         ctx: &Context,
/// #         command: &ApplicationCommandInteraction,
/// #     ) -> Result<(), InvocationError> {
/// #         unimplemented!()
/// #     }
/// # }
///
/// #[derive(Debug, SubCommand)]
/// struct TestSubCommandTwo;
/// # #[async_trait]
/// # impl ApplicationCommandInteractionHandler for TestSubCommandTwo {
/// #     async fn invoke(
/// #         &self,
/// #         ctx: &Context,
/// #         command: &ApplicationCommandInteraction,
/// #     ) -> Result<(), InvocationError> {
/// #         unimplemented!()
/// #     }
/// # }
///
/// // 2. Create an enum with a variant for each subcommand and derive the Command and
/// // ApplicationCommandInteractionHandler traits:
///
/// /// A test command to show subcommands
/// #[derive(Debug, Command, ApplicationCommandInteractionHandler)]
/// #[name = "test"]
/// enum TestCommand {
///     /// The first subcommand
///     #[name = "one"]
///     One(TestSubCommandOne),
///
///     /// The second subcommand
///     #[name = "two"]
///     Two(TestSubCommandTwo),
/// }
/// ```
///
/// If there is a lot of shared behaviour between the subcommands, you may wish to directly implement
/// the [`ApplicationCommandInteractionHandler`] trait for this [`Command`] enum rather than for each
/// [`SubCommand`].
///
/// To organize subcommands into groups, see the [`SubCommandGroup`] trait
pub trait SubCommand: Sized {
    /// Try to parse this from a command option
    fn parse(option: Option<&ApplicationCommandInteractionDataOption>) -> Result<Self, ParseError>;
    /// Register any sub options for this subcommand
    fn register_sub_options(
        option: &mut CreateApplicationCommandOption,
    ) -> &mut CreateApplicationCommandOption;
}

/// This trait provides the functions necessary to parse and register a subcommand group for a slash
/// command.
///
/// For most use cases:
/// ```
/// # use serenity::async_trait;
/// # use serenity::prelude::*;
/// # use serenity::model::interactions::*;
/// # use serenity::model::prelude::application_command::*;
/// # use slashies::*;
/// # use slashies_macros::*;
/// # use slashies::parsable::*;
/// #
/// // 1. Create the "leaf" subcommands as normal (see SubCommand docs)
/// # #[derive(Debug, SubCommand)]
/// # struct TestSubCommandOne;
/// # #[async_trait]
/// # impl ApplicationCommandInteractionHandler for TestSubCommandOne {
/// #     async fn invoke(
/// #         &self,
/// #         ctx: &Context,
/// #         command: &ApplicationCommandInteraction,
/// #     ) -> Result<(), InvocationError> {
/// #         unimplemented!()
/// #     }
/// # }
/// # #[derive(Debug, SubCommand)]
/// # struct TestSubCommandTwo;
/// # #[async_trait]
/// # impl ApplicationCommandInteractionHandler for TestSubCommandTwo {
/// #     async fn invoke(
/// #         &self,
/// #         ctx: &Context,
/// #         command: &ApplicationCommandInteraction,
/// #     ) -> Result<(), InvocationError> {
/// #         unimplemented!()
/// #     }
/// # }
/// # #[derive(Debug, SubCommand)]
/// # struct TestSubCommandThree;
/// # #[async_trait]
/// # impl ApplicationCommandInteractionHandler for TestSubCommandThree {
/// #     async fn invoke(
/// #         &self,
/// #         ctx: &Context,
/// #         command: &ApplicationCommandInteraction,
/// #     ) -> Result<(), InvocationError> {
/// #         unimplemented!()
/// #     }
/// # }
/// // 2. Create an enum with a variant for each subcommand in the subcommand group and derive the
/// // SubCommandGroup and ApplicationCommandInteractionHandler traits:
///
/// /// A test command to show subcommands
/// #[derive(Debug, SubCommandGroup, ApplicationCommandInteractionHandler)]
/// #[name = "test sub command group"]
/// enum TestSubCommandGroup {
///     /// The first subcommand
///     #[name = "one"]
///     One(TestSubCommandOne),
///
///     /// The second subcommand
///     #[name = "two"]
///     Two(TestSubCommandTwo),
/// }
///
/// // 3. Create an enum with a variant for each subcommand / subcommand group and derive the
/// // Command and ApplicationCommandInteractionHandler traits:
///
/// /// A test command to show subcommands
/// #[derive(Debug, Command, ApplicationCommandInteractionHandler)]
/// #[name = "test"]
/// enum TestCommand {
///     /// The subcommand group
///     #[subcommandgroup]
///     #[name = "group"]
///     Group(TestSubCommandGroup),
///
///     /// A regular subcommand
///     #[name = "three"]
///     Three(TestSubCommandThree),
/// }
/// ```
/// Note that you can mix subcommands and subcommand groups in a command as in the example above.
pub trait SubCommandGroup: Sized {
    /// Try to parse this from a command option
    fn parse(option: Option<&ApplicationCommandInteractionDataOption>) -> Result<Self, ParseError>;
    /// Register any sub options for this subcommand group
    fn register_sub_options(
        option: &mut CreateApplicationCommandOption,
    ) -> &mut CreateApplicationCommandOption;
}

/// This trait provides a function to receive and respond to slash command interactions.
///
/// Typically you will want to respond using [`create_interaction_response`] - see the [`serenity`]
/// docs for more info.
///
/// [`create_interaction_response`]: serenity::model::interactions::application_command::ApplicationCommandInteraction::create_interaction_response
/// ```
/// # use serenity::async_trait;
/// # use serenity::prelude::*;
/// # use serenity::model::interactions::*;
/// # use serenity::model::prelude::application_command::*;
/// # use slashies::*;
/// # use slashies::parsable::*;
/// # struct HelloCommand {
/// #     user: UserInput,
/// # }
///
/// #[async_trait]
/// impl ApplicationCommandInteractionHandler for HelloCommand {
///     async fn invoke(
///         &self,
///         ctx: &Context,
///         command: &ApplicationCommandInteraction,
///     ) -> Result<(), InvocationError> {
///         let nickname = self.user.member.as_ref().map(|pm| pm.nick.as_ref()).flatten();
///         let greeting = if let Some(nick) = nickname {
///             format!("Hello {} aka {}", self.user.user.name, nick)
///         } else {
///             format!("Hello {}", self.user.user.name)
///         };
///         command
///             .create_interaction_response(&ctx.http, |response| {
///                 response
///                     .kind(InteractionResponseType::ChannelMessageWithSource)
///                     .interaction_response_data(|message| message.content(greeting))
///             })
///             .await
///             .map_err(|_| InvocationError)
///     }
/// }
/// ```
#[async_trait]
pub trait ApplicationCommandInteractionHandler {
    /// Invoke the command
    async fn invoke(
        &self,
        ctx: &Context,
        command: &ApplicationCommandInteraction,
    ) -> Result<(), InvocationError>;
}

/// This trait provides a function to receive and respond to message component interactions.
///
/// TODO: more docs around handler map context
#[async_trait]
pub trait MessageComponentInteractionHandler {
    /// Handle the message component interaction
    async fn invoke(
        &mut self,
        ctx: &Context,
        interaction: &MessageComponentInteraction,
        original_message: &mut Message,
    );
}

/// This trait should be derived for an enum with a variant for each command.
/// This will implement the boilerplate to:
/// - Parse an interaction into a specific command based on the command name
/// - Delegate the invocation of a command to the specific enum variant
///
/// ```
/// # use slashies::*;
/// # use slashies_macros::*;
/// # use serenity::prelude::*;
/// # use serenity::model::prelude::application_command::*;
/// # #[derive(Debug, Command)]
/// # #[name = "greet"]
/// # /// Greet a user
/// # struct HelloCommand;
/// # #[serenity::async_trait]
/// # impl ApplicationCommandInteractionHandler for HelloCommand {
/// #     async fn invoke(
/// #         &self,
/// #         ctx: &Context,
/// #         command: &ApplicationCommandInteraction,
/// #     ) -> Result<(), InvocationError> {
/// #     unimplemented!()
/// #     }
/// # }
/// #[derive(Debug, Commands)]
/// enum BotCommands {
///     Hello(HelloCommand),
/// }
/// ```
#[async_trait]
pub trait Commands: Sized {
    /// Parse an interaction into a specific command
    fn parse(ctx: &Context, command: &ApplicationCommandInteraction) -> Result<Self, ParseError>;

    /// Invoke the command
    async fn invoke(
        &self,
        ctx: &Context,
        command_interaction: &ApplicationCommandInteraction,
    ) -> Result<(), InvocationError>;
}

/// Register a set of commands (either globally or to a specific guild)
///
/// Note: register each [`Command`] here rather than the [`Commands`] enum. This gives you the
/// flexibility to have some commands registered globally and others registered only in specific
/// guilds.
///
/// Examples:
/// ```no_run
/// # use slashies::*;
/// # use slashies_macros::*;
/// # use serenity::async_trait;
/// # use serenity::prelude::*;
/// # use serenity::model::prelude::*;
/// # use serenity::model::prelude::application_command::*;
/// # /// Greet a user
/// # #[derive(Debug, Command)]
/// # #[name = "greet"]
/// # struct HelloCommand;
/// # #[async_trait]
/// # impl ApplicationCommandInteractionHandler for HelloCommand {
/// #    async fn invoke(
/// #        &self,
/// #        ctx: &Context,
/// #        command: &ApplicationCommandInteraction,
/// #    ) -> Result<(), InvocationError> {
/// #     unimplemented!()
/// #     }
/// # }
/// # /// Another command
/// # #[derive(Debug, Command)]
/// # #[name = "next"]
/// # struct NextCommand;
/// # #[async_trait]
/// # impl ApplicationCommandInteractionHandler for NextCommand {
/// #    async fn invoke(
/// #        &self,
/// #        ctx: &Context,
/// #        command: &ApplicationCommandInteraction,
/// #    ) -> Result<(), InvocationError> {
/// #     unimplemented!()
/// #     }
/// # }
/// # /// Another command
/// # #[derive(Debug, Command)]
/// # #[name = "other"]
/// # struct OtherCommand;
/// # #[async_trait]
/// # impl ApplicationCommandInteractionHandler for OtherCommand {
/// #    async fn invoke(
/// #        &self,
/// #        ctx: &Context,
/// #        command: &ApplicationCommandInteraction,
/// #    ) -> Result<(), InvocationError> {
/// #         unimplemented!()
/// #     }
/// # }
/// # async fn test(ctx: Context) {
/// let guild_id = Some(GuildId(0));
///
/// // Register a command to a guild
/// register_commands!(&ctx, guild_id, [HelloCommand]);
///
/// // Register multiple commands
/// register_commands!(&ctx, guild_id, [HelloCommand, NextCommand, OtherCommand]);
///
/// // Register a global command
/// register_commands!(&ctx, None, [HelloCommand]);
/// # }
/// ```
#[macro_export]
macro_rules! register_commands {
    ($ctx:expr, $guild_id:expr, [$($cmdType:ty),+]) => {{
        if let Some(guild_id) = $guild_id {
            serenity::model::prelude::GuildId::set_application_commands(&guild_id, &$ctx.http, |commands_builder| {
                commands_builder
                $(
                    .create_application_command(|command| <$cmdType as slashies::Command>::register(command))
                )*
            })
            .await
        } else {
            serenity::model::interactions::application_command::ApplicationCommand::set_global_application_commands(&$ctx.http, |commands_builder| {
                commands_builder
                $(
                    .create_application_command(|command| <$cmdType as slashies::Command>::register(command))
                )*
            })
            .await
        }
    }};
}
