use serenity::{
    async_trait,
    client::Context,
    model::interactions::{
        application_command::ApplicationCommandInteraction, InteractionResponseType,
    },
};
use slash_helper::{ApplicationCommandInteractionHandler, Command, InvocationError, SubCommand};
use slash_helper_macros::Command;

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

#[derive(Debug)]
pub struct TestSubCommandUnit;

impl SubCommand for TestSubCommandUnit {
    fn parse(
        _option: Option<&serenity::model::interactions::application_command::ApplicationCommandInteractionDataOption>,
    ) -> Result<Self, slash_helper::ParseError> {
        Ok(Self {})
    }

    fn register_sub_options(
        option: &mut serenity::builder::CreateApplicationCommandOption,
    ) -> &mut serenity::builder::CreateApplicationCommandOption {
        option
    }
}

#[derive(Debug)]
pub struct TestSubCommandFields {
    /// an int option on a sub command
    sub_cmd_int: i64,
}

impl SubCommand for TestSubCommandFields {
    fn parse(
        option: Option<&serenity::model::interactions::application_command::ApplicationCommandInteractionDataOption>,
    ) -> Result<Self, slash_helper::ParseError> {
        let options: std::collections::HashMap<String, serenity::model::interactions::application_command::ApplicationCommandInteractionDataOption> = option
            .ok_or(slash_helper::ParseError::MissingOption)?
            .options
            .iter()
            .map(|option| (option.name.clone(), option.clone()))
            .collect();
        let sub_cmd_int = <i64 as slash_helper::parsable::ParsableCommandOption>::parse_from(
            options.get("sub_cmd_int"),
        )?;
        Ok(Self { sub_cmd_int })
    }

    fn register_sub_options(
        option: &mut serenity::builder::CreateApplicationCommandOption,
    ) -> &mut serenity::builder::CreateApplicationCommandOption {
        option
        .create_sub_option(|sub_option| {
            sub_option.kind(serenity::model::interactions::application_command::ApplicationCommandOptionType::Integer)
                .name("sub_cmd_int")
                .description("an int option on a sub command")
        })
    }
}
