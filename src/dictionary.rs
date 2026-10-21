use rand::seq::IteratorRandom;
use std::{
    fs,
    io::{BufRead, BufReader, Error},
};

#[derive(Debug, Default)]
pub struct Dictionary {
    words: Vec<String>,
}

impl Dictionary {
    // How many words must pass before a chance to spawn
    // another punctuation character is rolled
    const PUNCTUATION_WORD_DISTANCE: usize = 4;
    const PUNCTUATION_CHARACTERS_APPENDED: &[char] = &['.', ',', ';'];

    pub fn from<T, K>(iter: T) -> Self
    where
        T: IntoIterator<Item = K>,
        K: Into<String>,
    {
        Self {
            words: iter.into_iter().map(Into::into).collect(),
        }
    }

    pub fn from_file(file_name: &str) -> Result<Self, Error> {
        let file = fs::File::open(file_name)?;
        let reader = BufReader::new(file);
        let words = reader.lines().collect::<Result<Vec<String>, _>>()?;

        Ok(Self::from(words))
    }

    fn get_random_words_iter(&self, how_many: usize) -> impl Iterator<Item = &str> {
        (0..how_many)
            .filter_map(|_| self.words.iter().choose(&mut rand::rng()))
            .map(|x| x.as_str())
    }

    pub fn get_random_words(&self, how_many: usize) -> Vec<&str> {
        self.get_random_words_iter(how_many).collect()
    }

    pub fn get_random_words_with_punctuation(&self, how_many: usize) -> Vec<String> {
        let mut must_capitalize = false;
        let mut words_accumulator = 0;
        let mut result = Vec::with_capacity(how_many);

        self.get_random_words_iter(how_many).for_each(|x| {
            let random_appended = Self::PUNCTUATION_CHARACTERS_APPENDED
                .iter()
                .choose(&mut rand::rng());

            if words_accumulator <= Self::PUNCTUATION_WORD_DISTANCE {
                words_accumulator += 1;
                result.push(x.to_string());
                return;
            }

            if let Some(random_appended) = Self::PUNCTUATION_CHARACTERS_APPENDED
                .iter()
                .choose(&mut rand::rng())
            {
                let mut word = x.to_string();
                word.push(*random_appended);
                words_accumulator = 0;
                result.push(word);
                return;
            }

            words_accumulator += 1;
            result.push(x.to_string());
            return;
        });

        result
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
