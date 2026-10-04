use rand::seq::IteratorRandom;
use std::{
    fs,
    io::{BufRead, BufReader, Error},
};

#[derive(Debug)]
pub struct Dictionary {
    words: Vec<String>,
}

impl Dictionary {
    pub fn from<T, K>(iter: T) -> Self
    where
        T: IntoIterator<Item = K>,
        K: Into<String>,
    {
        Dictionary {
            words: iter.into_iter().map(Into::into).collect(),
        }
    }

    pub fn from_file(file_name: &str) -> Result<Self, Error> {
        let file = fs::File::open(file_name)?;
        let reader = BufReader::new(file);
        let words = reader.lines().collect::<Result<Vec<String>, _>>()?;

        Ok(Self::from(words))
    }

    pub fn get_random_words(&self, how_many: usize) -> Vec<&str> {
        (0..how_many)
            .filter_map(|_| self.words.iter().choose(&mut rand::rng()))
            .collect::<Vec<&String>>()
            .iter()
            .map(|&x| x.as_str())
            .collect::<Vec<&str>>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn initialize_dictionary_from_str_array() {
        let dictionary = Dictionary::from(["a", "bcd", "ef"]);
        assert_eq!(dictionary.words, vec!["a", "bcd", "ef"])
    }

    #[test]
    fn initialize_dictionary_from_string_vector() {
        let dictionary = Dictionary::from(vec!["a", "bcd", "ef"]);
        assert_eq!(dictionary.words, vec!["a", "bcd", "ef"])
    }

    #[test]
    fn initialize_dictionary_from_empty_vector() {
        let dictionary = Dictionary::from(Vec::<String>::new());
        assert_eq!(dictionary.words, Vec::<String>::new())
    }

    #[test]
    fn get_zero_words_from_good_dictionary() {
        let dictionary = Dictionary::from(vec!["a", "bcd", "ef"]);
        let words = dictionary.get_random_words(0);
        assert_eq!(words, Vec::<&str>::new());
    }

    #[test]
    fn get_zero_words_from_bad_dictionary() {
        let dictionary = Dictionary::from(Vec::<String>::new());
        let words = dictionary.get_random_words(0);
        println!("{}", dictionary.words.len());
        assert_eq!(words, Vec::<&str>::new());
    }

    #[test]
    fn get_one_word_from_good_dictionary() {
        let dictionary = Dictionary::from(vec!["a", "bcd", "ef"]);
        let words = dictionary.get_random_words(1);

        let dictionary_words: HashSet<String> = dictionary.words.clone().into_iter().collect();
        let random_words: HashSet<String> = words.into_iter().map(|x| x.to_string()).collect();

        assert!(random_words.is_subset(&dictionary_words));
    }

    #[test]
    fn get_one_word_from_bad_dictionary() {
        let dictionary = Dictionary::from(Vec::<String>::new());
        let words = dictionary.get_random_words(1);

        let dictionary_words: HashSet<String> = dictionary.words.clone().into_iter().collect();
        let random_words: HashSet<String> = words.into_iter().map(|x| x.to_string()).collect();

        assert!(random_words.is_subset(&dictionary_words));
    }

    #[test]
    fn get_more_words_from_dictionary_than_it_has() {
        const WORDS: usize = 5;

        let dictionary = Dictionary::from(vec!["a", "bcd", "ef"]);
        assert!(dictionary.words.len() < WORDS);
        let words = dictionary.get_random_words(WORDS);
        assert!(words.len() == WORDS);

        let dictionary_words: HashSet<String> = dictionary.words.clone().into_iter().collect();
        let random_words: HashSet<String> = words.into_iter().map(|x| x.to_string()).collect();

        assert!(random_words.is_subset(&dictionary_words));
    }
}
