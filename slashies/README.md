# A simple way to create slash commands for Discord bots

Slashies helps to reduce the boiler plate code needed to create slashcommands for a Discord bot.
It is built on top of [Serenity](https://github.com/serenity-rs/serenity). It focuses on providing traits that you can derive using the
macros crate for most straightforward use cases, but gives you the escape hatch of implementing
these traits yourself if you want to do something more complex.

Make sure to read the [Discord documentation](https://discord.com/developers/docs/interactions/application-commands)
on slash commands to understand the general concepts like interactions.

With Slashies, you can create a slash command in four easy steps:
```
# use slashies::*;
# use slashies::parsable::*;
# use slashies_macros::*;
# use serenity::async_trait;
# use serenity::prelude::*;
# use serenity::model::prelude::*;
# use serenity::model::prelude::application_command::*;
// 1. Create a struct representing the arguments for the command and derive/implement the
// Command trait

/// Greet a user
#[derive(Debug, Command)]
#[name = "greet"]
struct HelloCommand {
    /// The user to greet
    user: UserInput,
}

// 2. Implement the ApplicationCommandInteractionHandler trait to define what happens when you
// call the command
#[async_trait]
impl ApplicationCommandInteractionHandler for HelloCommand {
   async fn invoke(
       &self,
       ctx: &Context,
       command: &ApplicationCommandInteraction,
   ) -> Result<(), InvocationError> {
       let nickname = self.user.member.as_ref().map(|pm| pm.nick.as_ref()).flatten();
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

// 3. Add the command to an enum that implements the Commands trait, representing all the
// commands for the bot
#[derive(Debug, Commands)]
enum BotCommands {
    Hello(HelloCommand),
}

// 4. Add the basic code to register the command via a macro and handle interactions
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
        register_commands!(&ctx, None, [HelloCommand])
            .expect("Unable to register commands");
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
```