use rusqlite::{params, Connection, OpenFlags};
use std::{collections::HashMap, path::Path};

#[derive(Debug, Clone)]
pub struct Movie {
    pub title: String,
    pub average_rating: f64,
    pub directors: Vec<String>,
    pub release_year: i64,
    pub genres: Vec<String>,
}

struct MovieRow {
    id: String,
    title: String,
    average_rating: f64,
    director: String,
    release_year: i64,
    genres: Vec<String>,
}

#[derive(Debug)]
pub struct MovieDatabase {
    pub imdb_sqlite_file: String,
}

const FETCH_RECOMMENDATIONS_WITH_MIN_RATING_STATEMENT: &'static str = "
SELECT titles.title_id, titles.primary_title, titles.premiered, titles.genres, ratings.rating, people.name
FROM titles
INNER JOIN ratings ON titles.title_id = ratings.title_id
INNER JOIN crew ON titles.title_id = crew.title_id
INNER JOIN people ON crew.person_id = people.person_id
WHERE crew.category = 'director'
AND titles.type = 'movie'
AND titles.title_id IN (
    SELECT titles.title_id FROM titles 
    INNER JOIN ratings ON titles.title_id = ratings.title_id
    INNER JOIN crew ON titles.title_id = crew.title_id
    WHERE titles.type = 'movie'
    AND titles.genres LIKE ?1
    AND ratings.rating >= ?2
    AND crew.category = 'director'
    ORDER BY random()
    LIMIT ?3
)";

const FETCH_RECOMMENDATIONS_STATEMENT: &'static str = "
SELECT titles.title_id, titles.primary_title, titles.premiered, titles.genres, ratings.rating, people.name
FROM titles
INNER JOIN ratings ON titles.title_id = ratings.title_id
INNER JOIN crew ON titles.title_id = crew.title_id
INNER JOIN people ON crew.person_id = people.person_id
WHERE crew.category = 'director'
AND titles.type = 'movie'
AND titles.title_id IN (
    SELECT titles.title_id FROM titles
    INNER JOIN crew ON titles.title_id = crew.title_id
    WHERE titles.type = 'movie'
    AND titles.genres LIKE ?1
    AND crew.category = 'director'
    ORDER BY random()
    LIMIT ?2
)";

impl MovieDatabase {
    pub fn get_movie_recommendations(
        &self,
        genre: &str,
        min_rating: &Option<f64>,
        num_recommendations: &Option<i64>,
    ) -> Vec<Movie> {
        let connection = Connection::open_with_flags(
            Path::new(&self.imdb_sqlite_file),
            OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_SHARED_CACHE,
        )
        .expect("Unable to open movies database");
        let movie_rows: Vec<MovieRow> = if let Some(min_rating) = min_rating {
            let mut statement = connection
                .prepare(FETCH_RECOMMENDATIONS_WITH_MIN_RATING_STATEMENT)
                .expect("Unable to prepare query");
            statement
                .query_map(
                    params![
                        format!("%{}%", genre),
                        min_rating.clone(),
                        num_recommendations.unwrap_or(3)
                    ],
                    |row| {
                        let genres: String = row.get(3).unwrap();
                        Ok(MovieRow {
                            id: row.get(0).unwrap(),
                            title: row.get(1).unwrap(),
                            release_year: row.get(2).unwrap(),
                            genres: genres.split(",").map(|s| s.to_owned()).collect(),
                            average_rating: row.get(4).unwrap(),
                            director: row.get(5).unwrap(),
                        })
                    },
                )
                .expect("Error executing query")
                .map(|result| result.expect("Error transforming result into movie"))
                .collect()
        } else {
            let mut statement = connection
                .prepare(FETCH_RECOMMENDATIONS_STATEMENT)
                .expect("Unable to prepare query");
            statement
                .query_map(
                    params![format!("%{}%", genre), num_recommendations.unwrap_or(3)],
                    |row| {
                        let genres: String = row.get(3).unwrap();
                        Ok(MovieRow {
                            id: row.get(0).unwrap(),
                            title: row.get(1).unwrap(),
                            release_year: row.get(2).unwrap(),
                            genres: genres.split(",").map(|s| s.to_owned()).collect(),
                            average_rating: row.get(4).unwrap(),
                            director: row.get(5).unwrap(),
                        })
                    },
                )
                .expect("Error executing query")
                .map(|result| result.expect("Error transforming result into movie row"))
                .collect()
        };
        movie_rows
            .into_iter()
            .map(|movie_row| (movie_row.id.clone(), movie_row))
            .fold(HashMap::new(), |mut map, (id, row)| {
                let movie = map.entry(id).or_insert(Movie {
                    title: row.title,
                    average_rating: row.average_rating,
                    directors: Vec::new(),
                    release_year: row.release_year,
                    genres: row.genres,
                });
                movie.directors.push(row.director);
                map
            })
            .into_iter()
            .map(|(_id, movie)| movie)
            .collect()
    }
}
