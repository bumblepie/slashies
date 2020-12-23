#[macro_use]
extern crate diesel;

mod counting;
mod database;
pub mod models;
pub mod schema;

use chrono::Utc;
use counting::count_line;
use lazy_static::lazy_static;
use models::{Haiku, HaikuLine};
use serenity::{
    client::{bridge::gateway::GatewayIntents, Context},
    framework::standard::{
        macros::{command, group, hook},
        Args, CommandResult,
    },
    framework::StandardFramework,
    model::channel::Message,
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

async fn send_haiku_embed(channel: ChannelId, haiku: &Haiku, haiku_id: i64, ctx: &Context) {
    let (authors, lines): (Vec<UserId>, Vec<String>) = haiku
        .lines
        .to_vec()
        .into_iter()
        .map(|line| (line.author, line.content.clone()))
        .unzip();
    let actual_channel = channel.to_channel(&ctx.http).await.unwrap();
    let members = actual_channel
        .guild()
        .unwrap()
        .members(&ctx.cache)
        .await
        .unwrap();
    let primary_author = members.iter().find(|member| member.user.id == authors[2]);
    lazy_static! {
        static ref BOT_USER_ID: String = env::var("DISCORD_USER_ID")
            .expect("Expected a user id in the environment")
            .parse()
            .expect("Invalid user id");
    }
    let primary_author_nickname = match primary_author {
        Some(author) => author.display_name().to_string(),
        None => "Unknown User".to_owned(),
    };
    let primary_author_icon = match primary_author {
        Some(author) => author.user.avatar_url(),
        None => None,
    };
    let primary_author_colour = match primary_author {
        Some(author) => author.colour(&ctx.cache).await,
        None => None,
    };
    let bot_member = ctx.cache.current_user().await;
    let bot_icon = bot_member.avatar_url();

    channel
        .send_message(&ctx.http, |msg| {
            msg.embed(|embed| {
                embed.title("A beautiful haiku has been created!");
                embed.description(lines.join("\n"));
                embed.url("https://github.com/bumblepie/haikubot");
                embed.color(primary_author_colour.unwrap_or_default());
                embed.timestamp(&haiku.timestamp);
                embed.footer(|footer| {
                    footer
                        .icon_url(bot_icon.unwrap_or(
                            "https://cdn.discordapp.com/embed/avatars/0.png".to_owned(),
                        ));
                    footer.text(format!("Haiku #{}", haiku_id));
                    footer
                });
                embed.author(|author| {
                    author.name(primary_author_nickname);
                    author
                        .icon_url(primary_author_icon.unwrap_or(
                            "https://cdn.discordapp.com/embed/avatars/0.png".to_owned(),
                        ));
                    author
                });
                embed
            });
            msg
        })
        .await
        .expect("Failed to send haiku msg");
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
    channel_messages[2] = Some(line);
    let haiku = match channel_messages {
        [Some(line_1), Some(line_2), Some(line_3)] => {
            let lines = [line_1.clone(), line_2.clone(), line_3.clone()];
            if count_line(&lines[0].content) == Ok(5)
                && count_line(&lines[1].content) == Ok(7)
                && count_line(&lines[2].content) == Ok(5)
            {
                let actual_channel = channel.to_channel(&ctx.http).await.unwrap();
                Some(Haiku {
                    lines,
                    timestamp: Utc::now(),
                    channel: channel,
                    server: actual_channel.guild().unwrap().guild_id,
                })
            } else {
                None
            }
        }
        _ => None,
    };
    if let Some(haiku) = haiku {
        let db_connection = database::establish_connection();
        let id = database::save_haiku(&haiku, &db_connection);
        send_haiku_embed(channel, &haiku, id, ctx).await;
    }
}

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

#[command]
async fn get(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let haiku_and_id = match (args.single(), msg.guild_id) {
        (Ok(id), Some(server_id)) => {
            let db_connection = database::establish_connection();
            database::get_haiku(server_id, id, &db_connection).map(|haiku| (id, haiku))
        }
        _ => None,
    };
    if let Some((id, haiku)) = haiku_and_id {
        send_haiku_embed(msg.channel_id, &haiku, id, ctx).await;
    }
    Ok(())
}
#[group]
#[commands(count, get)]
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
        .group(&GENERAL_GROUP);
    let mut client = Client::builder(&token)
        .framework(framework)
        .add_intent(GatewayIntents::GUILDS)
        .add_intent(GatewayIntents::GUILD_MEMBERS)
        .add_intent(GatewayIntents::GUILD_PRESENCES)
        .add_intent(GatewayIntents::GUILD_MESSAGES)
        .await
        .expect("Err creating client");

    {
        let mut data = client.data.write().await;
        data.insert::<HaikuTracker>(Arc::new(RwLock::new(HashMap::new())));
    }

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}
