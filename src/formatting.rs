use std::{collections::HashSet, env};

use crate::models::Haiku;
use chrono::{DateTime, Utc};
use lazy_static::lazy_static;
use serenity::{builder::CreateEmbed, client::Context, model::id::UserId, utils::Color};

pub struct EmbedData {
    haiku_lines: Vec<String>,
    haiku_id: i64,
    haiku_timestamp: DateTime<Utc>,
    bot_icon_url: Option<String>,
    unique_authors: Vec<String>,
    primary_author_color: Option<Color>,
    primary_author_icon: Option<String>,
}

pub async fn to_embed_data(id: i64, haiku: &Haiku, ctx: &Context) -> EmbedData {
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

pub fn format_haiku_embed(embed_data: EmbedData, embed: &mut CreateEmbed) -> &mut CreateEmbed {
    let author_string = embed_data.unique_authors.join(", ");
    let author_icon_url = embed_data
        .primary_author_icon
        .clone()
        .unwrap_or("https://cdn.discordapp.com/embed/avatars/0.png".to_owned());
    embed.title("A beautiful haiku has been created!");
    embed.description(embed_data.haiku_lines.join("\n"));
    embed.url("https://github.com/bumblepie/haikubot-rs");
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
}
