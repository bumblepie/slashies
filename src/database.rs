use crate::models::*;
use crate::Haiku;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use rand::Rng;
use serenity::model::id::GuildId;
use std::convert::TryFrom;
use std::env;

pub fn establish_connection() -> PgConnection {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url).expect(&format!("Error connecting to {}", database_url))
}

pub fn save_haiku(haiku: &Haiku, database_connection: &PgConnection) -> i64 {
    use crate::schema::haikus;
    let new_haiku = NewHaikuDTO::from(haiku);
    let haiku: HaikuDTO = diesel::insert_into(haikus::table)
        .values(&new_haiku)
        .get_result(database_connection)
        .expect("Error saving haiku");
    haiku.id
}

pub fn get_haiku(
    server_id: GuildId,
    haiku_id: i64,
    database_connection: &PgConnection,
) -> Option<(i64, Haiku)> {
    use crate::schema::haikus::dsl::*;
    let results = haikus
        .filter(server.eq(i64::try_from(*server_id.as_u64()).unwrap()))
        .filter(id.eq(haiku_id))
        .load::<HaikuDTO>(database_connection)
        .expect("Error fetching haiku");
    results.into_iter().next().map(|dto| dto.into())
}

pub fn get_random_haiku(
    server_id: GuildId,
    database_connection: &PgConnection,
) -> Option<(i64, Haiku)> {
    use crate::schema::haikus::dsl::*;
    let count = haikus
        .filter(server.eq(i64::try_from(*server_id.as_u64()).unwrap()))
        .count()
        .get_result::<i64>(database_connection)
        .expect("Error fetching haiku");
    for _ in 0..10 {
        let haiku_id = rand::thread_rng().gen_range(0, count);
        if let Some(haiku) = get_haiku(server_id, haiku_id, database_connection) {
            return Some(haiku);
        }
    }
    None
}
