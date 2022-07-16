use serenity::{
    async_trait,
    client::{Context, EventHandler},
    model::{
        channel::PartialChannel,
        guild::Role,
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
use slashies::{
    parsable::{Mentionable, UserInput},
    register_commands, ApplicationCommandInteractionHandler, Commands, InvocationError,
};
use slashies_macros::{Command, Commands};
use std::env::VarError;

/// Create a madlib
#[derive(Debug, Command)]
#[name = "madlib"]
struct MadlibCommand {
    /// An integer
    integer: i64,

    /// A percentage
    percentage: f64,

    /// A plural noun
    plural_noun: String,

    /// A verb (optional)
    verb: Option<String>,

    /// A boolean
    boolean: bool,

    /// A channel
    channel: PartialChannel,

    /// A role
    role: Role,

    /// A user or a role
    mentionable: Mentionable,

    /// A user
    user: UserInput,
}

#[async_trait]
impl ApplicationCommandInteractionHandler for MadlibCommand {
    async fn invoke(
        &self,
        ctx: &Context,
        command: &ApplicationCommandInteraction,
    ) -> Result<(), InvocationError> {
        let username = self
            .user
            .member
            .as_ref()
            .map(|pm| pm.nick.as_ref())
            .flatten()
            .unwrap_or(&self.user.user.name);
        let verb = self.verb.clone().unwrap_or("love".to_owned());
        let channel_name = self
            .channel
            .name
            .clone()
            .unwrap_or("the basement".to_owned());
        if let Some(ref invoker) = command.member {
            let mut sentences = Vec::new();
            sentences.push(format!(
                "Once upon a time, the lands of Discordia were conquered by the dark lord {}.",
                invoker.display_name(),
            ));
            sentences.push(format!(
                "Using their army of {}, they ruled with an iron fist.",
                self.role.name,
            ));
            sentences.push(format!(
                "Luckily, there was a wizard of great renown, {}!",
                username,
            ));
            sentences.push(format!(
                "They had {} {}, the most of any wizard, which of course gave them their immense power.",
                self.integer, self.plural_noun,
            ));
            sentences.push(format!(
                "The great wizard knew their power alone would not be enough, so they sought counsel from the wise ones in {}. This was less useful than they had hoped.",
                channel_name,
            ));
            sentences.push(format!(
                "The only {}, who taught {} how to {}.",
                match self.mentionable {
                    Mentionable::User(ref user) => format!("one of any use was {}", user.user.name),
                    Mentionable::Role(ref role) => format!("ones of any use were {}", role.name),
                },
                username,
                verb,
            ));
            sentences.push(format!(
                "After fighting through the army of {}, the time had come to confront {}.",
                self.role.name,
                invoker.display_name(),
            ));
            sentences.push(format!("\"I would advise against this,\" said {}'s funny robot companion, \"Our chances of success are {}%.\"", username, self.percentage));
            sentences.push(format!("\"Silly robot,\" replied {}, \"It's 50/50, we either do it or we don't. Besides, we can just {}!\"",
                username, verb,
            ));
            sentences.push(match self.boolean {
                true => format!("After a hard fought battle, {} emerged victorious and the land of Discordia was saved!", username),
                false => format!("Unfortunately, trying to {} was somewhat ineffective, and the wizard was defeated by the dark lord {}.", verb, invoker.display_name()),
            });

            command
                .create_interaction_response(&ctx.http, |response| {
                    response
                        .kind(InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|message| message.content(sentences.join(" ")))
                })
                .await
                .map_err(|_| InvocationError)
        } else {
            // Just ignore non-guild messages
            Ok(())
        }
    }
}

#[derive(Debug, Commands)]
enum BotCommands {
    Madlib(MadlibCommand),
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
                .expect("Invalid test guild id id")
        });
        let guild_id = match guild_id {
            Ok(id) => Some(id),
            Err(VarError::NotPresent) => None,
            _ => panic!("Invalid guild id provided at $TEST_GUILD_ID"),
        };
        let commands = register_commands!(&ctx, guild_id, [MadlibCommand])
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
