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
        .map(|word| count_word(word))
        .collect::<Result<Vec<usize>, Uncountable>>()?;
    Ok(word_syllables.into_iter().sum())
}
fn count_word(word: &str) -> Result<usize, Uncountable> {
    match count_word_strict(word.to_owned()) {
        Ok(count) => Ok(count),
        // Try again after trimming punctuation
        Err(_) => count_word_strict(word.trim_matches(|c: char| !c.is_alphanumeric()).to_owned()),
    }
}

#[cached(size = 1000)]
fn count_word_strict(word: String) -> Result<usize, Uncountable> {
    lazy_static! {
        static ref WORD_REGEX: Regex = Regex::new(r"^[\w']+$").unwrap();
    }
    if !WORD_REGEX.is_match(&word) {
        Err(Uncountable)
    } else {
        let file = File::open("cmu_dict.txt").unwrap();
        let reader = BufReader::new(file);
        let line: Option<String> = reader
            .lines()
            .filter_map(Result::ok)
            .filter(|line| line.starts_with(&format!("{} ", word.to_uppercase())))
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

pub fn is_haiku(lines: &[String]) -> bool {
    count_line(&lines[0]) == Ok(5)
        && count_line(&lines[1]) == Ok(7)
        && count_line(&lines[2]) == Ok(5)
}

pub fn is_haiku_single(line: &str) -> Result<Option<[String; 3]>, Uncountable> {
    let mut syllable_count = 0;
    let mut lines = [Vec::new(), Vec::new(), Vec::new()];
    for word in line.split_whitespace() {
        syllable_count += count_word(word)?;
        match syllable_count {
            x if x <= 5 => lines[0].push(word.to_owned()),
            x if x <= 12 => lines[1].push(word.to_owned()),
            _ => lines[2].push(word.to_owned()),
        }
    }
    let lines = [lines[0].join(" "), lines[1].join(" "), lines[2].join(" ")];
    if is_haiku(&lines) {
        Ok(Some(lines))
    } else {
        Ok(None)
    }
}

#[cfg(test)]
mod test {
    use super::{count_line, count_word, is_haiku, is_haiku_single, Uncountable};

    #[test]
    fn test_count_word() {
        assert_eq!(count_word("ABUNDANT"), Ok(3));
        assert_eq!(count_word("abundant"), Ok(3));
        assert_eq!(count_word("abUNdaNT"), Ok(3));
        assert_eq!(count_word("a"), Ok(1));
        assert_eq!(count_word("A"), Ok(1));
        assert_eq!(count_word("Don't"), Ok(1));
        assert_eq!(count_word("'Allo"), Ok(2));
        assert_eq!(count_word("Allo"), Err(Uncountable));
        assert_eq!(count_word("X Y Z"), Err(Uncountable));
        assert_eq!(count_word("XYZ"), Err(Uncountable));
        assert_eq!(count_word("#$&^%&"), Err(Uncountable));
    }

    #[test]
    fn test_count_line() {
        assert_eq!(count_line("A B C"), Ok(3));
        assert_eq!(count_line("Abundant haiku"), Ok(5));
        assert_eq!(count_line("Uses a dictionary"), Ok(7));
        assert_eq!(count_line("Database to come"), Ok(5));
        assert_eq!(count_line("'Allo 'allo"), Ok(4));
        assert_eq!(count_line("'Hello there'"), Ok(3));
        assert_eq!(count_line("\"Hello there.\" said General Kenobi."), Ok(10));
    }

    #[test]
    fn test_haiku_single() {
        assert_eq!(
            is_haiku_single(
                "The last winter leaves Clinging to the black branches Explode into birds"
            ),
            Ok(Some([
                "The last winter leaves".to_owned(),
                "Clinging to the black branches".to_owned(),
                "Explode into birds".to_owned()
            ]))
        );
        assert_eq!(
            is_haiku_single(
                "The last winter leaves, clinging to the black branches, explode into birds."
            ),
            Ok(Some([
                "The last winter leaves,".to_owned(),
                "clinging to the black branches,".to_owned(),
                "explode into birds.".to_owned()
            ]))
        );
        assert_eq!(
            is_haiku_single(
                "The last spring leaves Clinging to the black branches Explode into birds"
            ),
            Ok(None)
        );
        assert_eq!(
            is_haiku_single(
                "The last ^%^$&^ leaves Clinging to the black branches Explode into birds"
            ),
            Err(Uncountable)
        );
    }

    #[test]
    fn test_haiku() {
        assert!(is_haiku(&[
            "The last winter leaves".to_owned(),
            "Clinging to the black branches".to_owned(),
            "Explode into birds".to_owned()
        ]));
        assert!(is_haiku(&[
            "The last 'winter' leaves.".to_owned(),
            "Clinging to the black branches.".to_owned(),
            "Explode into birds".to_owned()
        ]));
        assert!(!is_haiku(&[
            "The last spring leaves".to_owned(),
            "Clinging to the black branches".to_owned(),
            "Explode into birds".to_owned()
        ]));
        assert!(!is_haiku(&[
            "The last $^%$^ leaves".to_owned(),
            "Clinging to the black branches".to_owned(),
            "Explode into birds".to_owned()
        ]));
    }
}
