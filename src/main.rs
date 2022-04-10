#[macro_use]
extern crate diesel;

mod commands;
mod counting;
mod database;
mod formatting;
pub mod models;
pub mod schema;

use chrono::{DateTime, Utc};
use commands::Commands;
use counting::{is_haiku, is_haiku_single};
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

// #[hook]
async fn on_message(ctx: &Context, msg: &Message) {
    let channel = msg.channel_id;
    let lines = msg.content.lines().map(|content| HaikuLine {
        author: msg.author.id,
        content: content.to_owned(),
    });
    for line in lines {
        on_haiku_line(ctx, channel, line).await;
    }
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

/// Search for a haiku, using a set of keywords separated by spaces
/// Returns up to five matching haiku from this server
// #[command]
// async fn search(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
//     let keywords = args.iter().collect::<Result<Vec<String>, _>>()?;

//     if let Some(server_id) = msg.guild_id {
//         let db_connection = database::establish_connection();
//         let search_results = database::search_haikus(server_id, keywords, &db_connection);
//         if !search_results.is_empty() {
//             let mut index = 0;
//             let (id, haiku) = search_results.get(index).unwrap();
//             let embed_data = to_embed_data(*id, &haiku, ctx).await;
//             let mut search_result_msg = msg
//                 .channel_id
//                 .send_message(&ctx.http, |msg| {
//                     msg.embed(|embed| format_haiku_embed(embed_data, embed));
//                     msg.content(format!(
//                         "Search result {}/{}",
//                         index + 1,
//                         search_results.len()
//                     ));
//                     msg
//                 })
//                 .await
//                 .expect("Failed to send search results");
//             search_result_msg
//                 .react(&ctx.http, ReactionType::Unicode("⬅️".to_owned()))
//                 .await
//                 .expect("Failed to add reaction to search results msg");
//             search_result_msg
//                 .react(&ctx.http, ReactionType::Unicode("➡️".to_owned()))
//                 .await
//                 .expect("Failed to add reaction to search results msg");
//             loop {
//                 if let Some(reaction) = search_result_msg
//                     .await_reaction(ctx)
//                     .timeout(Duration::from_secs(300))
//                     .await
//                 {
//                     if let Some((new_index, (id, haiku))) =
//                         match reaction.as_inner_ref().emoji.as_data().as_str() {
//                             "➡️" => {
//                                 let new_index = index + 1;
//                                 search_results.get(new_index).map(|x| (new_index, x))
//                             }
//                             "⬅️" => {
//                                 if let Some(new_index) = index.checked_sub(1) {
//                                     search_results.get(new_index).map(|x| (new_index, x))
//                                 } else {
//                                     None
//                                 }
//                             }
//                             _ => None,
//                         }
//                     {
//                         let embed_data = to_embed_data(*id, &haiku, ctx).await;
//                         search_result_msg
//                             .edit(&ctx.http, |msg| {
//                                 msg.embed(|embed| format_haiku_embed(embed_data, embed));
//                                 msg.content(format!(
//                                     "Search result {}/{}",
//                                     new_index + 1,
//                                     search_results.len()
//                                 ));
//                                 msg
//                             })
//                             .await
//                             .expect("Failed to edit search results message");
//                         index = new_index;
//                         reaction
//                             .as_inner_ref()
//                             .delete(&ctx.http)
//                             .await
//                             .expect("Unable to delete reaction");
//                     }
//                 } else {
//                     break;
//                 }
//             }
//         } else {
//             msg.reply(&ctx.http, "No matching haiku found")
//                 .await
//                 .expect("Failed to send search results msg");
//         }
//     }
//     Ok(())
// }

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
        if let Interaction::ApplicationCommand(command_interaction) = interaction {
            Commands::parse(&ctx, &command_interaction)
                .expect("Failed to parse command")
                .invoke(&ctx, &command_interaction)
                .await
                .expect("Failed to invoke command");
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
        let commands = commands::register_commands(&ctx)
            .await
            .expect("Unable to register commands");
        println!(
            "I now have the following guild slash commands: {:#?}",
            commands
        );

        // let guild_command =
        //     ApplicationCommand::create_global_application_command(&ctx.http, |command| {
        //         command.name("wonderful_command").description("An amazing command")
        //     })
        //     .await;

        // println!("I created the following global slash command: {:#?}", guild_command);
    }
}

#[tokio::main]
async fn main() {
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    let application_id = env::var("DISCORD_USER_ID")
        .expect("Expected a user id in the environment")
        .parse::<u64>()
        .expect("Invalid user id");

    // let framework = StandardFramework::new()
    //     .configure(|c| c.on_mention(Some(UserId(user_id))).prefix(""))
    //     .normal_message(on_message)
    //     .group(&GENERAL_GROUP)
    //     .help(&MY_HELP);
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
    }

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}
