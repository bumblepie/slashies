use super::schema::haikus;
use chrono::{DateTime, NaiveDateTime, Utc};
use serenity::model::id::{ChannelId, GuildId, UserId};
use std::convert::TryFrom;

#[derive(Debug, Clone)]
pub struct Haiku {
    pub lines: [HaikuLine; 3],
    pub timestamp: DateTime<Utc>,
    pub channel: ChannelId,
    pub server: GuildId,
}

#[derive(Debug, Clone)]
pub struct HaikuLine {
    pub author: UserId,
    pub content: String,
}

#[derive(Debug, Queryable)]
pub struct HaikuDTO {
    pub id: i64,
    pub channel: i64,
    pub server: i64,
    pub timestamp: NaiveDateTime,
    pub author_0: i64,
    pub author_1: i64,
    pub author_2: i64,
    pub message_0: String,
    pub message_1: String,
    pub message_2: String,
}

impl Into<(i64, Haiku)> for HaikuDTO {
    fn into(self) -> (i64, Haiku) {
        (
            self.id,
            Haiku {
                lines: [
                    HaikuLine {
                        content: self.message_0,
                        author: UserId::from(u64::try_from(self.author_0).unwrap()),
                    },
                    HaikuLine {
                        content: self.message_1,
                        author: UserId::from(u64::try_from(self.author_1).unwrap()),
                    },
                    HaikuLine {
                        content: self.message_2,
                        author: UserId::from(u64::try_from(self.author_2).unwrap()),
                    },
                ],
                timestamp: DateTime::from_utc(self.timestamp, Utc),
                channel: ChannelId::from(u64::try_from(self.channel).unwrap()),
                server: GuildId::from(u64::try_from(self.server).unwrap()),
            },
        )
    }
}

#[derive(Insertable)]
#[table_name = "haikus"]
pub struct NewHaikuDTO {
    pub channel: i64,
    pub server: i64,
    pub timestamp: NaiveDateTime,
    pub author_0: i64,
    pub author_1: i64,
    pub author_2: i64,
    pub message_0: String,
    pub message_1: String,
    pub message_2: String,
}

impl From<&Haiku> for NewHaikuDTO {
    fn from(haiku: &Haiku) -> Self {
        NewHaikuDTO {
            channel: i64::try_from(*haiku.channel.as_u64()).unwrap(),
            server: i64::try_from(*haiku.server.as_u64()).unwrap(),
            timestamp: haiku.timestamp.naive_utc(),
            author_0: i64::try_from(*haiku.lines[0].author.as_u64()).unwrap(),
            author_1: i64::try_from(*haiku.lines[0].author.as_u64()).unwrap(),
            author_2: i64::try_from(*haiku.lines[0].author.as_u64()).unwrap(),
            message_0: haiku.lines[0].content.clone(),
            message_1: haiku.lines[1].content.clone(),
            message_2: haiku.lines[2].content.clone(),
        }
    }
}
