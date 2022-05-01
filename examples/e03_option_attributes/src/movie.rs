#[derive(Debug)]
pub struct Movie {
    pub title: String,
    pub average_rating: f64,
    pub director: String,
    pub release_year: i64,
    pub genres: Vec<String>,
}

pub struct MovieDatabase {}

impl MovieDatabase {
    pub fn get_movie_recommendations(
        &self,
        _genre: &str,
        _min_rating: &Option<f64>,
        _num_recommendations: &Option<i64>,
    ) -> Vec<Movie> {
        vec![Movie {
            title: "Star Wars".to_owned(),
            average_rating: 4.8,
            director: "George Lucas".to_owned(),
            release_year: 1978,
            genres: vec!["Sci-Fi".to_owned()],
        }]
    }
}
