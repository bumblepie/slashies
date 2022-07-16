use serenity::{
    async_trait,
    client::{Context, EventHandler},
    model::{
        id::GuildId,
        interactions::{
            application_command::ApplicationCommandInteraction, Interaction,
            InteractionResponseType,
        },
        prelude::Ready,
    },
    prelude::GatewayIntents,
    Client,
};
use slash_helper::{
    parsable::UserInput, register_commands, ApplicationCommandInteractionHandler, Commands,
    InvocationError,
};
use slash_helper_macros::{Command, Commands};
use std::env::VarError;

/// Greet a user
#[derive(Debug, Command)]
#[name = "greet"]
struct HelloCommand {
    /// The user to greet
    user: UserInput,
}

#[async_trait]
impl ApplicationCommandInteractionHandler for HelloCommand {
    async fn invoke(
        &self,
        ctx: &Context,
        command: &ApplicationCommandInteraction,
    ) -> Result<(), InvocationError> {
        let nickname = self
            .user
            .member
            .as_ref()
            .map(|pm| pm.nick.as_ref())
            .flatten();
        let greeting = if let Some(nick) = nickname {
            format!("Hello {} aka {}", self.user.user.name, nick)
        } else {
            format!("Hello {}", self.user.user.name)
        };
        command
            .create_interaction_response(&ctx.http, |response| {
                response
                    .kind(InteractionResponseType::ChannelMessageWithSource)
                    .interaction_response_data(|message| message.content(greeting))
            })
            .await
            .map_err(|_| InvocationError)
    }
}

#[derive(Debug, Commands)]
enum BotCommands {
    Hello(HelloCommand),
}

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        match interaction {
            Interaction::ApplicationCommand(command_interaction) => {
                BotCommands::parse(&ctx, &command_interaction)
                    .expect("Failed to parse command")
                    .invoke(&ctx, &command_interaction)
                    .await
                    .expect("Failed to invoke command");
            }
            _ => (),
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
        let guild_id = std::env::var("TEST_GUILD_ID").map(|id| {
            id.parse()
                .map(|id| GuildId(id))
                .expect(&format!("Invalid test guild id {}", id))
        });
        let guild_id = match guild_id {
            Ok(id) => Some(id),
            Err(VarError::NotPresent) => None,
            _ => panic!("Invalid guild id provided at $TEST_GUILD_ID"),
        };
        let commands = register_commands!(&ctx, guild_id, [HelloCommand])
            .expect("Unable to register commands");
        println!(
            "Registered {} commands {}",
            commands.len(),
            match guild_id {
                Some(id) => format!("for guild_id: {}", id),
                None => "globally".to_owned(),
            },
        );
    }
}

#[tokio::main]
async fn main() {
    let token = std::env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    let application_id = std::env::var("DISCORD_USER_ID")
        .expect("Expected a user id in the environment")
        .parse::<u64>()
        .expect("Invalid user id");
    let mut client = Client::builder(&token, GatewayIntents::empty())
        .event_handler(Handler)
        .application_id(application_id)
        .await
        .expect("Err creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}
