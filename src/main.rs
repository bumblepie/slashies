#[macro_use]
extern crate diesel;

mod counting;
mod database;
mod formatting;
pub mod models;
pub mod schema;

use chrono::{DateTime, Utc};
use counting::{count_line, is_haiku, is_haiku_single};
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
use std::{collections::HashMap, sync::Arc};
use std::{env, time::Duration};

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

/// Count the number of syllables in a given phrase
/// This bot uses the CMU dictionary http://www.speech.cs.cmu.edu/cgi-bin/cmudict so some words might be uncountable
// #[command]
// async fn count(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
//     match count_line(&args.message()) {
//         Ok(syllables) => {
//             msg.reply(
//                 &ctx.http,
//                 format!("Message '{}' has {} syllables", args.message(), syllables),
//             )
//             .await?;
//         }
//         Err(_) => {
//             msg.reply(&ctx.http, "Message is not countable").await?;
//         }
//     }
//     Ok(())
// }

/// Fetch a specific haiku from this server by its id
// #[command]
// async fn get(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
//     let haiku_and_id = match (args.single(), msg.guild_id) {
//         (Ok(id), Some(server_id)) => {
//             let db_connection = database::establish_connection();
//             database::get_haiku(server_id, id, &db_connection)
//         }
//         _ => None,
//     };
//     if let Some((id, haiku)) = haiku_and_id {
//         let embed_data = to_embed_data(id, &haiku, ctx).await;
//         msg.channel_id
//             .send_message(&ctx.http, |msg| {
//                 msg.embed(|embed| format_haiku_embed(embed_data, embed));
//                 msg
//             })
//             .await
//             .expect("Failed to send haiku msg");
//     }
//     Ok(())
// }

/// Fetch a random haiku from this server
// #[command]
// async fn random(ctx: &Context, msg: &Message) -> CommandResult {
//     let haiku_and_id = if let Some(server_id) = msg.guild_id {
//         let db_connection = database::establish_connection();
//         database::get_random_haiku(server_id, &db_connection)
//     } else {
//         None
//     };
//     if let Some((id, haiku)) = haiku_and_id {
//         let embed_data = to_embed_data(id, &haiku, ctx).await;
//         msg.channel_id
//             .send_message(&ctx.http, |msg| {
//                 msg.embed(|embed| format_haiku_embed(embed_data, embed));
//                 msg
//             })
//             .await
//             .expect("Failed to send haiku msg");
//     }
//     Ok(())
// }

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

/// Show how long since the bot was last restarted
// #[command]
// async fn uptime(ctx: &Context, msg: &Message) -> CommandResult {
//     let data = ctx.data.read().await;
//     let uptime_start_lock = data
//         .get::<UptimeStart>()
//         .expect("Expected HaikuTracker in TypeMap")
//         .clone();
//     let uptime = Utc::now().signed_duration_since(uptime_start_lock);
//     let days = uptime.num_days();
//     let uptime = uptime - chrono::Duration::days(days);
//     let hrs = uptime.num_hours();
//     let uptime = uptime - chrono::Duration::hours(hrs);
//     let mins = uptime.num_minutes();

//     msg.reply(
//         &ctx.http,
//         format!("Uptime: {} days, {} hours, {} minutes", days, hrs, mins),
//     )
//     .await
//     .expect("Could not send uptime message");
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
        if let Interaction::ApplicationCommand(command) = interaction {
            let content = match command.data.name.as_str() {
                "ping" => "Hey, I'm alive!".to_string(),
                // "id" => {
                //     let options = command
                //         .data
                //         .options
                //         .get(0)
                //         .expect("Expected user option")
                //         .resolved
                //         .as_ref()
                //         .expect("Expected user object");

                //     if let ApplicationCommandInteractionDataOptionValue::User(user, _member) =
                //         options
                //     {
                //         format!("{}'s id is {}", user.tag(), user.id)
                //     } else {
                //         "Please provide a valid user".to_string()
                //     }
                // }
                _ => "not implemented :(".to_string(),
            };

            if let Err(why) = command
                .create_interaction_response(&ctx.http, |response| {
                    response
                        .kind(InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|message| message.content(content))
                })
                .await
            {
                println!("Cannot respond to slash command: {}", why);
            }
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);

        let guild_id = env::var("TEST_GUILD_ID")
            .expect("Expected a test guild id in the environment")
            .parse()
            .expect("Invalid test guild id id");
        let guild_id = GuildId(guild_id);

        let commands = GuildId::set_application_commands(&guild_id, &ctx.http, |commands| {
            commands.create_application_command(|command| {
                command.name("ping").description("A ping command")
            })
            // .create_application_command(|command| {
            //     command.name("id").description("Get a user id").create_option(|option| {
            //         option
            //             .name("id")
            //             .description("The user to lookup")
            //             .kind(ApplicationCommandOptionType::User)
            //             .required(true)
            //     })
            // })
            // .create_application_command(|command| {
            //     command
            //         .name("welcome")
            //         .description("Welcome a user")
            //         .create_option(|option| {
            //             option
            //                 .name("user")
            //                 .description("The user to welcome")
            //                 .kind(ApplicationCommandOptionType::User)
            //                 .required(true)
            //         })
            //         .create_option(|option| {
            //             option
            //                 .name("message")
            //                 .description("The message to send")
            //                 .kind(ApplicationCommandOptionType::String)
            //                 .required(true)
            //                 .add_string_choice(
            //                     "Welcome to our cool server! Ask me if you need help",
            //                     "pizza",
            //                 )
            //                 .add_string_choice("Hey, do you want a coffee?", "coffee")
            //                 .add_string_choice(
            //                     "Welcome to the club, you're now a good person. Well, I hope.",
            //                     "club",
            //                 )
            //                 .add_string_choice(
            //                     "I hope that you brought a controller to play together!",
            //                     "game",
            //                 )
            //         })
            // })
            // .create_application_command(|command| {
            //     command
            //         .name("numberinput")
            //         .description("Test command for number input")
            //         .create_option(|option| {
            //             option
            //                 .name("int")
            //                 .description("An integer from 5 to 10")
            //                 .kind(ApplicationCommandOptionType::Integer)
            //                 .min_int_value(5)
            //                 .max_int_value(10)
            //                 .required(true)
            //         })
            //         .create_option(|option| {
            //             option
            //                 .name("number")
            //                 .description("A float from -3.3 to 234.5")
            //                 .kind(ApplicationCommandOptionType::Number)
            //                 .min_number_value(-3.3)
            //                 .max_number_value(234.5)
            //                 .required(true)
            //         })
            // })
        })
        .await;

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
