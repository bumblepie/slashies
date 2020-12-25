#[macro_use]
extern crate diesel;

mod counting;
mod database;
pub mod models;
pub mod schema;

use chrono::{DateTime, Utc};
use counting::{count_line, is_haiku, is_haiku_single};
use lazy_static::lazy_static;
use models::{Haiku, HaikuLine};
use serenity::{
    client::{bridge::gateway::GatewayIntents, Context},
    framework::standard::help_commands,
    framework::standard::macros::help,
    framework::standard::{
        macros::{command, group, hook},
        Args, CommandGroup, CommandResult, HelpOptions,
    },
    framework::StandardFramework,
    model::channel::Message,
    model::prelude::*,
    prelude::RwLock,
    prelude::TypeMapKey,
    utils::Color,
    Client,
};
use std::{collections::HashMap, collections::HashSet, sync::Arc};
use std::{env, time::Duration};

struct HaikuTracker;
impl TypeMapKey for HaikuTracker {
    type Value = Arc<RwLock<HashMap<ChannelId, [Option<HaikuLine>; 3]>>>;
}

struct UptimeStart;
impl TypeMapKey for UptimeStart {
    type Value = DateTime<Utc>;
}

#[hook]
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

struct EmbedData {
    haiku_lines: Vec<String>,
    haiku_id: i64,
    haiku_timestamp: DateTime<Utc>,
    bot_icon_url: Option<String>,
    unique_authors: Vec<String>,
    primary_author_color: Option<Color>,
    primary_author_icon: Option<String>,
}

async fn to_embed(id: i64, haiku: &Haiku, ctx: &Context) -> EmbedData {
    let (authors, lines): (Vec<UserId>, Vec<String>) = haiku
        .lines
        .to_vec()
        .into_iter()
        .map(|line| (line.author, line.content.clone()))
        .unzip();
    let members = ctx
        .cache
        .guild_channel(haiku.channel)
        .await
        .unwrap()
        .members(&ctx.cache)
        .await
        .unwrap();
    let primary_author = members.iter().find(|member| member.user.id == authors[0]);
    lazy_static! {
        static ref BOT_USER_ID: String = env::var("DISCORD_USER_ID")
            .expect("Expected a user id in the environment")
            .parse()
            .expect("Invalid user id");
    }
    let primary_author_icon = match primary_author {
        Some(author) => author.user.avatar_url(),
        None => None,
    };
    let primary_author_color = match primary_author {
        Some(author) => author.colour(&ctx.cache).await,
        None => None,
    };

    // Deduplicate retaining order
    let mut unique_authors = authors.clone();
    let mut unique_authors_set = HashSet::new();
    unique_authors.retain(|x| unique_authors_set.insert(x.clone()));

    let unique_authors = unique_authors
        .into_iter()
        .map(
            |author| match members.iter().find(|member| member.user.id == author) {
                Some(author) => author.display_name().to_string(),
                None => "Unknown User".to_owned(),
            },
        )
        .collect();

    let bot_member = ctx.cache.current_user().await;
    let bot_icon_url = bot_member.avatar_url();
    EmbedData {
        haiku_lines: lines,
        haiku_id: id,
        haiku_timestamp: haiku.timestamp,
        bot_icon_url,
        unique_authors,
        primary_author_color,
        primary_author_icon,
    }
}

async fn edit_haiku_embed(
    message: &mut Message,
    haiku: &Haiku,
    haiku_id: i64,
    content: Option<String>,
    ctx: &Context,
) -> serenity::Result<()> {
    let embed_data = to_embed(haiku_id, haiku, ctx).await;
    message
        .edit(&ctx.http, |msg| {
            let author_string = embed_data.unique_authors.join(", ");
            let author_icon_url = embed_data
                .primary_author_icon
                .clone()
                .unwrap_or("https://cdn.discordapp.com/embed/avatars/0.png".to_owned());
            msg.embed(|embed| {
                embed.title("A beautiful haiku has been created!");
                embed.description(embed_data.haiku_lines.join("\n"));
                embed.url("https://github.com/bumblepie/haikubot");
                embed.color(embed_data.primary_author_color.unwrap_or_default());
                embed.timestamp(&embed_data.haiku_timestamp);
                embed.footer(|footer| {
                    footer.icon_url(
                        embed_data
                            .bot_icon_url
                            .unwrap_or("https://cdn.discordapp.com/embed/avatars/0.png".to_owned()),
                    );
                    footer.text(format!("Haiku #{}", embed_data.haiku_id));
                    footer
                });
                embed.author(|author| {
                    author.name(author_string);
                    author.icon_url(author_icon_url);
                    author
                });
                embed
            });
            if let Some(content) = content {
                msg.content(content);
            }
            msg
        })
        .await
}

async fn send_haiku_embed(
    channel: ChannelId,
    haiku: &Haiku,
    haiku_id: i64,
    content: Option<String>,
    ctx: &Context,
) -> serenity::Result<Message> {
    let embed_data = to_embed(haiku_id, haiku, ctx).await;
    channel
        .send_message(&ctx.http, |msg| {
            let author_string = embed_data.unique_authors.join(", ");
            let author_icon_url = embed_data
                .primary_author_icon
                .clone()
                .unwrap_or("https://cdn.discordapp.com/embed/avatars/0.png".to_owned());
            msg.embed(|embed| {
                embed.title("A beautiful haiku has been created!");
                embed.description(embed_data.haiku_lines.join("\n"));
                embed.url("https://github.com/bumblepie/haikubot");
                embed.color(embed_data.primary_author_color.unwrap_or_default());
                embed.timestamp(&embed_data.haiku_timestamp);
                embed.footer(|footer| {
                    footer.icon_url(
                        embed_data
                            .bot_icon_url
                            .unwrap_or("https://cdn.discordapp.com/embed/avatars/0.png".to_owned()),
                    );
                    footer.text(format!("Haiku #{}", embed_data.haiku_id));
                    footer
                });
                embed.author(|author| {
                    author.name(author_string);
                    author.icon_url(author_icon_url);
                    author
                });
                embed
            });
            if let Some(content) = content {
                msg.content(content);
            }
            msg
        })
        .await
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
        send_haiku_embed(channel, &haiku, id, None, ctx)
            .await
            .expect("Failed to send haiku msg");
    }
}

/// Count the number of syllables in a given phrase
/// This bot uses the CMU dictionary http://www.speech.cs.cmu.edu/cgi-bin/cmudict so some words might be uncountable
#[command]
async fn count(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    match count_line(&args.message()) {
        Ok(syllables) => {
            msg.reply(
                &ctx.http,
                format!("Message '{}' has {} syllables", args.message(), syllables),
            )
            .await?;
        }
        Err(_) => {
            msg.reply(&ctx.http, "Message is not countable").await?;
        }
    }
    Ok(())
}

/// Fetch a specific haiku from this server by its id
#[command]
async fn get(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let haiku_and_id = match (args.single(), msg.guild_id) {
        (Ok(id), Some(server_id)) => {
            let db_connection = database::establish_connection();
            database::get_haiku(server_id, id, &db_connection)
        }
        _ => None,
    };
    if let Some((id, haiku)) = haiku_and_id {
        send_haiku_embed(msg.channel_id, &haiku, id, None, ctx)
            .await
            .expect("Failed to send haiku msg");
    }
    Ok(())
}

/// Fetch a random haiku from this server
#[command]
async fn random(ctx: &Context, msg: &Message) -> CommandResult {
    let haiku_and_id = if let Some(server_id) = msg.guild_id {
        let db_connection = database::establish_connection();
        database::get_random_haiku(server_id, &db_connection)
    } else {
        None
    };
    if let Some((id, haiku)) = haiku_and_id {
        send_haiku_embed(msg.channel_id, &haiku, id, None, ctx)
            .await
            .expect("Failed to send haiku msg");
    }
    Ok(())
}

/// Search for a haiku, using a set of keywords separated by spaces
/// Returns up to five matching haiku from this server
#[command]
async fn search(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let keywords = args.iter().collect::<Result<Vec<String>, _>>()?;

    if let Some(server_id) = msg.guild_id {
        let db_connection = database::establish_connection();
        let search_results = database::search_haikus(server_id, keywords, &db_connection);
        if !search_results.is_empty() {
            let mut index = 0;
            let (id, haiku) = search_results.get(index).unwrap();
            let mut search_result_msg = send_haiku_embed(
                msg.channel_id,
                haiku,
                *id,
                Some(format!(
                    "Search result {}/{}",
                    index + 1,
                    search_results.len()
                )),
                ctx,
            )
            .await
            .expect("Failed to send search results");
            search_result_msg
                .react(&ctx.http, ReactionType::Unicode("⬅️".to_owned()))
                .await
                .expect("Failed to add reaction to search results msg");
            search_result_msg
                .react(&ctx.http, ReactionType::Unicode("➡️".to_owned()))
                .await
                .expect("Failed to add reaction to search results msg");
            loop {
                if let Some(reaction) = search_result_msg
                    .await_reaction(ctx)
                    .timeout(Duration::from_secs(300))
                    .await
                {
                    if let Some((new_index, (id, haiku))) =
                        match reaction.as_inner_ref().emoji.as_data().as_str() {
                            "➡️" => {
                                let new_index = index + 1;
                                search_results.get(new_index).map(|x| (new_index, x))
                            }
                            "⬅️" => {
                                if let Some(new_index) = index.checked_sub(1) {
                                    search_results.get(new_index).map(|x| (new_index, x))
                                } else {
                                    None
                                }
                            }
                            _ => None,
                        }
                    {
                        edit_haiku_embed(
                            &mut search_result_msg,
                            haiku,
                            *id,
                            Some(format!(
                                "Search result {}/{}",
                                new_index + 1,
                                search_results.len()
                            )),
                            ctx,
                        )
                        .await
                        .expect("Failed to edit search results message");
                        index = new_index;
                        reaction
                            .as_inner_ref()
                            .delete(&ctx.http)
                            .await
                            .expect("Unable to delete reaction");
                    }
                } else {
                    break;
                }
            }
        } else {
            msg.reply(&ctx.http, "No matching haiku found")
                .await
                .expect("Failed to send search results msg");
        }
    }
    Ok(())
}

/// Show how long since the bot was last restarted
#[command]
async fn uptime(ctx: &Context, msg: &Message) -> CommandResult {
    let data = ctx.data.read().await;
    let uptime_start_lock = data
        .get::<UptimeStart>()
        .expect("Expected HaikuTracker in TypeMap")
        .clone();
    let uptime = Utc::now().signed_duration_since(uptime_start_lock);
    let days = uptime.num_days();
    let uptime = uptime - chrono::Duration::days(days);
    let hrs = uptime.num_hours();
    let uptime = uptime - chrono::Duration::hours(hrs);
    let mins = uptime.num_minutes();

    msg.reply(
        &ctx.http,
        format!("Uptime: {} days, {} hours, {} minutes", days, hrs, mins),
    )
    .await
    .expect("Could not send uptime message");
    Ok(())
}

#[help]
async fn my_help(
    context: &Context,
    msg: &Message,
    args: Args,
    help_options: &'static HelpOptions,
    groups: &[&'static CommandGroup],
    owners: HashSet<UserId>,
) -> CommandResult {
    let _ = help_commands::with_embeds(context, msg, args, help_options, groups, owners).await;
    Ok(())
}

#[group]
#[commands(count, get, random, search, uptime)]
struct General;

#[tokio::main]
async fn main() {
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    let user_id = env::var("DISCORD_USER_ID")
        .expect("Expected a user id in the environment")
        .parse()
        .expect("Invalid user id");

    let framework = StandardFramework::new()
        .configure(|c| c.on_mention(Some(UserId(user_id))).prefix(""))
        .normal_message(on_message)
        .group(&GENERAL_GROUP)
        .help(&MY_HELP);
    let mut client = Client::builder(&token)
        .framework(framework)
        .add_intent(GatewayIntents::GUILDS)
        .add_intent(GatewayIntents::GUILD_MEMBERS)
        .add_intent(GatewayIntents::GUILD_PRESENCES)
        .add_intent(GatewayIntents::GUILD_MESSAGES)
        .add_intent(GatewayIntents::GUILD_MESSAGE_REACTIONS)
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
