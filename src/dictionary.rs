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
            .map(|_| self.words.iter().choose(&mut rand::rng()).unwrap())
            .collect::<Vec<&String>>()
            .iter()
            .map(|&x| x.as_str())
            .collect::<Vec<&str>>()
    }
}
