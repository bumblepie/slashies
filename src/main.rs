#[macro_use]
extern crate diesel;

mod commands;
mod counting;
mod database;
mod formatting;
pub mod models;
pub mod schema;

use chrono::{DateTime, Utc};
use commands::{
    count::CountCommand, gethaiku::GetHaikuCommand, random::RandomHaikuCommand,
    search::SearchCommand, test::TestCommand, test_sub::TestSubCommands, uptime::UptimeCommand,
    Commands,
};
use counting::{is_haiku, is_haiku_single};
use dashmap::DashMap;
use formatting::{format_haiku_embed, to_embed_data};
use models::{Haiku, HaikuLine};
use serenity::{
    async_trait,
    client::{bridge::gateway::GatewayIntents, Context, EventHandler},
    model::interactions::Interaction,
    model::prelude::*,
    prelude::RwLock,
    prelude::TypeMapKey,
    Client,
};
use slash_helper::{register_commands, MessageComponentInteractionHandler};
use std::env;
use std::{collections::HashMap, sync::Arc};

struct HaikuTracker;
impl TypeMapKey for HaikuTracker {
    type Value = Arc<RwLock<HashMap<ChannelId, [Option<HaikuLine>; 3]>>>;
}

struct UptimeStart;
impl TypeMapKey for UptimeStart {
    type Value = DateTime<Utc>;
}

struct MessageComponentInteractionHandlers;
impl TypeMapKey for MessageComponentInteractionHandlers {
    type Value = DashMap<InteractionId, Box<dyn MessageComponentInteractionHandler + Send + Sync>>;
}

async fn on_haiku_line(ctx: &Context, channel: ChannelId, line: HaikuLine) {
    let data_read = ctx.data.read().await;
    let tracker_lock = data_read
        .get::<HaikuTracker>()
        .expect("Expected HaikuTracker in TypeMap")
        .clone();
    let mut tracker = tracker_lock.write().await;
    let channel_messages = tracker.entry(channel).or_insert([None, None, None]);
    channel_messages[0] = channel_messages[1].clone();
    channel_messages[1] = channel_messages[2].clone();
    channel_messages[2] = Some(line.clone());
    let haiku = if let Ok(Some(lines)) = is_haiku_single(&line.content) {
        let guild_id = ctx.cache.guild_channel(channel).await.unwrap().guild_id;
        let author = line.author;
        let lines = [
            HaikuLine {
                author,
                content: lines[0].clone(),
            },
            HaikuLine {
                author,
                content: lines[1].clone(),
            },
            HaikuLine {
                author,
                content: lines[2].clone(),
            },
        ];
        Some(Haiku {
            lines,
            timestamp: Utc::now(),
            channel: channel,
            server: guild_id,
        })
    } else {
        match channel_messages {
            [Some(line_1), Some(line_2), Some(line_3)] => {
                let lines = [line_1.clone(), line_2.clone(), line_3.clone()];
                let line_contents = [
                    line_1.content.clone(),
                    line_2.content.clone(),
                    line_3.content.clone(),
                ];
                if is_haiku(&line_contents) {
                    let guild_id = ctx.cache.guild_channel(channel).await.unwrap().guild_id;
                    Some(Haiku {
                        lines,
                        timestamp: Utc::now(),
                        channel: channel,
                        server: guild_id,
                    })
                } else {
                    None
                }
            }
            _ => None,
        }
    };
    if let Some(haiku) = haiku {
        let db_connection = database::establish_connection();
        let id = database::save_haiku(&haiku, &db_connection);
        let embed_data = to_embed_data(id, &haiku, ctx).await;
        channel
            .send_message(&ctx.http, |msg| {
                msg.embed(|embed| format_haiku_embed(embed_data, embed));
                msg
            })
            .await
            .expect("Failed to send haiku msg");
    }
}

// #[help]
// async fn my_help(
//     context: &Context,
//     msg: &Message,
//     args: Args,
//     help_options: &'static HelpOptions,
//     groups: &[&'static CommandGroup],
//     owners: HashSet<UserId>,
// ) -> CommandResult {
//     let _ = help_commands::with_embeds(context, msg, args, help_options, groups, owners).await;
//     Ok(())
// }

// #[group]
// #[commands(count, get, random, search, uptime)]
// struct General;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        match interaction {
            Interaction::ApplicationCommand(command_interaction) => {
                Commands::parse(&ctx, &command_interaction)
                    .expect("Failed to parse command")
                    .invoke(&ctx, &command_interaction)
                    .await
                    .expect("Failed to invoke command");
            }
            Interaction::MessageComponent(component_interaction) => {
                if let Some(ref original_interaction) = component_interaction.message.interaction {
                    let data = ctx.data.read().await;
                    let handlers = data
                        .get::<MessageComponentInteractionHandlers>()
                        .expect("Expected Handlers in TypeMap");
                    let mut handler = handlers
                        .get_mut(&original_interaction.id)
                        .expect("No handler found for interaction");
                    handler
                        .invoke(
                            &ctx,
                            &component_interaction,
                            &mut component_interaction.message.clone(),
                        )
                        .await;
                }
            }
            _ => (),
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
        let guild_id = env::var("TEST_GUILD_ID")
            .expect("Expected a test guild id in the environment")
            .parse()
            .map(|id| GuildId(id))
            .expect("Invalid test guild id id");
        let commands = register_commands!(
            &ctx,
            Some(guild_id),
            [
                UptimeCommand,
                CountCommand,
                GetHaikuCommand,
                RandomHaikuCommand,
                SearchCommand,
                TestCommand,
                TestSubCommands
            ]
        )
        .expect("Unable to register commands");
        println!(
            "I now have the following guild slash commands: {:#?}",
            commands
        );
    }

    async fn message(&self, ctx: Context, msg: Message) {
        let channel = msg.channel_id;
        let lines = msg.content.lines().map(|content| HaikuLine {
            author: msg.author.id,
            content: content.to_owned(),
        });
        for line in lines {
            on_haiku_line(&ctx, channel, line).await;
        }
    }
}

#[tokio::main]
async fn main() {
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    let application_id = env::var("DISCORD_USER_ID")
        .expect("Expected a user id in the environment")
        .parse::<u64>()
        .expect("Invalid user id");
    let mut client = Client::builder(&token)
        // .framework(framework)
        .event_handler(Handler)
        .application_id(application_id)
        .intents(
            GatewayIntents::GUILDS
                | GatewayIntents::GUILD_MEMBERS
                | GatewayIntents::GUILD_PRESENCES
                | GatewayIntents::GUILD_MESSAGES
                | GatewayIntents::GUILD_MESSAGE_REACTIONS,
        )
        .await
        .expect("Err creating client");

    {
        let mut data = client.data.write().await;
        data.insert::<HaikuTracker>(Arc::new(RwLock::new(HashMap::new())));
        data.insert::<UptimeStart>(Utc::now());
        data.insert::<MessageComponentInteractionHandlers>(DashMap::new());
    }

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}
