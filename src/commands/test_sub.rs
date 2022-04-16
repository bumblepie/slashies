use serenity::{
    async_trait,
    client::Context,
    model::{
        channel::PartialChannel,
        guild::{PartialMember, Role},
        interactions::{
            application_command::ApplicationCommandInteraction, InteractionResponseType,
        },
        prelude::User,
    },
};
use slash_helper::{
    parsable::Mentionable, ApplicationCommandInteractionHandler, Command, InvocationError,
    SubCommand,
};
use slash_helper_macros::{Command, SubCommand};

/// Sub commmand test
#[derive(Debug, Command)]
#[name = "test_sub"]
pub enum TestSubCommands {
    /// Sub commmand with no options
    #[name = "unit"]
    Unit(TestSubCommandUnit),
    #[name = "fields"]
    /// Sub command with options
    Fields(TestSubCommandFields),
}
pub const TEST_SUB_COMMAND_NAME: &'static str = "test_sub";

#[async_trait]
impl ApplicationCommandInteractionHandler for TestSubCommands {
    async fn invoke(
        &self,
        ctx: &Context,
        command: &ApplicationCommandInteraction,
    ) -> Result<(), InvocationError> {
        println!("{:?}", self);
        command
            .create_interaction_response(&ctx.http, |response| {
                response
                    .kind(InteractionResponseType::ChannelMessageWithSource)
                    .interaction_response_data(|data| {
                        data.content("Test command complete, see logs for received command details")
                    })
            })
            .await
            .expect("Failed to send test command response");
        Ok(())
    }
}

#[derive(Debug, SubCommand)]
pub struct TestSubCommandUnit;

#[allow(dead_code)]
#[derive(Debug, SubCommand)]
pub struct TestSubCommandFields {
    /// a string option
    string_opt: String,
    /// a non-required string option
    maybe_string_opt: Option<String>,
    /// an int option
    int_opt: i64,
    /// a non-required int option
    maybe_int_opt: Option<i64>,
    /// a bool option
    bool_opt: bool,
    /// a non-required bool option
    maybe_bool_opt: Option<bool>,
    /// a number option
    num_opt: f64,
    /// a non-required num option
    maybe_num_opt: Option<f64>,
    /// a user option
    user_opt: (User, Option<PartialMember>),
    /// a non-required user option
    maybe_user_opt: Option<(User, Option<PartialMember>)>,
    /// a role option
    role_opt: Role,
    /// a non-required role option
    maybe_role_opt: Option<Role>,
    /// a mentionable option
    mentionable_opt: Mentionable,
    /// a non-required role option
    maybe_mentionable_opt: Option<Mentionable>,
    /// a channel option
    channel_opt: PartialChannel,
    /// a non-required role option
    maybe_channel_opt: Option<PartialChannel>,
}
