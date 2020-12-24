use crate::models::*;
use crate::Haiku;
use diesel::pg::PgConnection;
use diesel::{pg::Pg, prelude::*};
use diesel_full_text_search::{
    plainto_tsquery, to_tsvector, ts_rank_cd, TsQuery, TsQueryExtensions, TsVectorExtensions,
};
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

fn get_search_query(
    keywords: &Vec<String>,
) -> Option<Box<dyn BoxableExpression<crate::schema::haikus::table, Pg, SqlType = TsQuery>>> {
    keywords
        .iter()
        .map(|kw| plainto_tsquery(kw.to_owned()))
        .fold(None, |query, next| match query {
            None => Some(Box::new(next)
                as Box<
                    dyn BoxableExpression<crate::schema::haikus::table, Pg, SqlType = TsQuery>,
                >),
            Some(query) => Some(Box::new(query.or(next))
                as Box<
                    dyn BoxableExpression<crate::schema::haikus::table, Pg, SqlType = TsQuery>,
                >),
        })
}

pub fn search_haikus(
    server_id: GuildId,
    keywords: Vec<String>,
    database_connection: &PgConnection,
) -> Vec<(i64, Haiku)> {
    use crate::schema::haikus::dsl::*;
    let search_fields = to_tsvector(message_0)
        .concat(to_tsvector(message_1))
        .concat(to_tsvector(message_2));
    let search_query = get_search_query(&keywords);
    if let Some(search_query) = search_query {
        let search_query_2 = get_search_query(&keywords).unwrap();
        let query = haikus
            .filter(server.eq(i64::try_from(*server_id.as_u64()).unwrap()))
            .filter(search_query.matches(search_fields))
            .order(ts_rank_cd(search_fields, search_query_2).desc())
            .limit(5);
        let result = query
            .load::<HaikuDTO>(database_connection)
            .expect("Error searching for haikus");
        result.into_iter().map(|dto| dto.into()).collect()
    } else {
        Vec::new()
    }
}
