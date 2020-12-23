mod counting;

use counting::count_line;
use serenity::{
    async_trait,
    client::{Context, EventHandler},
    framework::standard::macros::hook,
    framework::StandardFramework,
    model::channel::Message,
    model::prelude::*,
    prelude::RwLock,
    prelude::TypeMapKey,
    Client,
};
use std::{collections::HashMap, sync::Arc};
use std::{collections::HashSet, env};

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.content == "ping" {
            if let Err(why) = msg.channel_id.say(&ctx.http, "Pong!").await {
                println!("Error sending message: {:?}", why);
            }
        }
    }

    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

#[derive(Debug, Clone)]
struct Haiku {
    lines: Vec<String>,
    authors: HashSet<UserId>,
}

#[derive(Debug, Clone)]
struct HaikuLine {
    author: UserId,
    content: String,
}
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
            let (lines, authors): (Vec<String>, HashSet<UserId>) = vec![line_1, line_2, line_3]
                .into_iter()
                .map(|line| (line.content.clone(), line.author))
                .unzip();
            if count_line(&lines[0]) == Ok(5)
                && count_line(&lines[1]) == Ok(7)
                && count_line(&lines[2]) == Ok(5)
            {
                Some(Haiku { lines, authors })
            } else {
                None
            }
        }
        _ => None,
    };
    if let Some(haiku) = haiku {
        channel
            .say(&ctx.http, format!("{:?}", haiku))
            .await
            .expect("Failed to send haiku msg");
    }
}

#[tokio::main]
async fn main() {
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    let user_id = env::var("DISCORD_USER_ID")
        .expect("Expected a user id in the environment")
        .parse()
        .expect("Invalid user id");

    let framework = StandardFramework::new()
        .configure(|c| c.on_mention(Some(UserId(user_id))).prefix(""))
        .normal_message(on_message);
    let mut client = Client::builder(&token)
        .event_handler(Handler)
        .framework(framework)
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
