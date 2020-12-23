use cached::proc_macro::cached;
use lazy_static::lazy_static;
use regex::Regex;
use std::io::BufRead;
use std::{fs::File, io::BufReader};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Uncountable;

pub fn count_line(line: &str) -> Result<usize, Uncountable> {
    let word_syllables = line
        .split_whitespace()
        .map(|word| count_word(word.to_owned()))
        .collect::<Result<Vec<usize>, Uncountable>>()?;
    Ok(word_syllables.into_iter().sum())
}

#[cached(size = 1000)]
fn count_word(word: String) -> Result<usize, Uncountable> {
    lazy_static! {
        static ref WORD_REGEX: Regex = Regex::new(r"^\w+$").unwrap();
    }
    if !WORD_REGEX.is_match(&word) {
        Err(Uncountable)
    } else {
        let file = File::open("cmu_dict.txt").unwrap();
        let reader = BufReader::new(file);
        let line: Option<String> = reader
            .lines()
            .filter_map(Result::ok)
            .filter(|line| line.starts_with(&word.to_uppercase()))
            .next();
        if let Some(line) = line {
            lazy_static! {
                // Match a whitespace char (don't include the word itself), then phoneme followed by stress
                static ref STRESS_REGEX: Regex = Regex::new(r"\s(?:[[:alpha:]]+([[:digit:]]))").unwrap();
            }
            Ok(STRESS_REGEX.captures_iter(&line).count())
        } else {
            Err(Uncountable)
        }
    }
}

#[cfg(test)]
mod test {
    use super::{count_line, count_word, Uncountable};

    #[test]
    fn test_count_word() {
        assert_eq!(count_word("ABUNDANT".to_owned()), Ok(3));
        assert_eq!(count_word("abundant".to_owned()), Ok(3));
        assert_eq!(count_word("abUNdaNT".to_owned()), Ok(3));
        assert_eq!(count_word("a".to_owned()), Ok(1));
        assert_eq!(count_word("A".to_owned()), Ok(1));
        assert_eq!(count_word("X Y Z".to_owned()), Err(Uncountable));
        assert_eq!(count_word("XYZ".to_owned()), Err(Uncountable));
        assert_eq!(count_word("#$&^%&".to_owned()), Err(Uncountable));
    }

    #[test]
    fn test_count_words() {
        assert_eq!(count_line("A B C"), Ok(3));
        assert_eq!(count_line("Abundant haiku"), Ok(5));
        assert_eq!(count_line("Uses a dictionary"), Ok(7));
        assert_eq!(count_line("Database to come"), Ok(5));
    }
}
