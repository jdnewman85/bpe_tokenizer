use std::collections::{HashMap, HashSet};
use std::io::{BufRead, BufReader, Read};
use std::{fs::File, path::Path};

use rayon::prelude::*;

mod pat;
mod bpe;

fn is_valid_bpe_char<T>(c: T) -> bool
where
    T: nom::AsChar + Copy, //TODO Or nom_unicode::IsChar?
{
    match c.as_char() {
        '!'..='~' | '¡'..='¬' | '®'..='ÿ' => true,
        _ => false,
    }
}

// Replaces bytes_to_unicode
fn create_bpe_char_encoder<T>() -> HashMap<T, char>
where
    T: nom::AsChar //TODO Or nom_unicode::IsChar?
        + Copy
        + std::hash::Hash
        + std::cmp::Eq
        + std::convert::From<u8>
        + std::fmt::Debug,
{
    let mut map: HashMap<T, char> = HashMap::new();

    let mut n: u32 = 0; //Current offset
    for i in 0..=255 {
        let is_valid = is_valid_bpe_char(i);
        let cs: char = match is_valid {
            true => char::from_u32(i as u32).unwrap(),
            false => {
                let map_to = 256 + n;
                n += 1;
                char::from_u32(map_to).unwrap()
            }
        };

        map.insert(i.into(), cs);
    }

    //    dbg!(&map);
    return map;
}

fn create_bpe_char_decoder<T>(encoder: HashMap<T, char>) -> HashMap<char, T>
where
    T: nom::AsChar //TODO Or nom_unicode::IsChar?
        + Copy
        + std::hash::Hash
        + std::cmp::Eq
        + std::convert::From<u8>
        + std::fmt::Debug,
{
    let decoder = encoder.into_iter().map(|(k, v)| (v, k)).collect();

    //    dbg!(&decoder);
    return decoder;
}

type BpeTokenEncoder = HashMap<String, u16>;
fn create_bpe_token_encoder<T>(filename: T) -> Result<BpeTokenEncoder, std::io::Error>
where
    T: AsRef<Path>,
{
    //TODO: Make more efficient
    let mut file = File::open(filename)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    let encoder: BpeTokenEncoder = serde_json::from_str(&contents)?;

    Ok(encoder)
}

type BpeTokenDecoder = HashMap<u16, String>;
fn create_bpe_token_decoder(
    encoder: HashMap<String, u16>,
) -> Result<BpeTokenDecoder, std::io::Error> {
    let decoder: BpeTokenDecoder = encoder.into_par_iter().map(|(k, v)| (v, k)).collect();

    Ok(decoder)
}

type BpePair = (String, String);
type BpeRanks = Vec<BpePair>;
fn create_bpe_ranks<T>(filename: T) -> Result<HashMap<BpePair, usize>, std::io::Error>
where
    T: AsRef<Path>,
{
    //TODO: Make more efficient
    let file = File::open(filename)?;
    let reader = BufReader::new(file);

    //First line is version comment, hence skip(1)
    let num_comment_lines = 1;
    let ranks: BpeRanks = reader
        .lines()
        .skip(num_comment_lines)
        .map(|line| {
            let line = line.unwrap();
            let mut split = line.split_whitespace();
            (
                split.next().unwrap().to_string(),
                split.next().unwrap().to_string(),
            )
        })
        .collect();

    let bpe_ranks = ranks
        .into_iter()
        .enumerate()
        .fold(HashMap::new(), |mut map, bpe_rank| {
            map.insert(bpe_rank.1, bpe_rank.0);
            map
        });

    Ok(bpe_ranks)
}

type BpeWord = Vec<String>;
fn generate_consecutive_pairs(word: &BpeWord) -> HashSet<BpePair> {
    word.windows(2)
        .map(|w| (w[0].to_owned(), w[1].to_owned()))
        .collect()
}

fn bpe_word_from_string<S>(s: S) -> BpeWord
where
    S: Into<String>,
{
    s.into().chars().map(|c| c.to_string()).collect()
}

pub trait TokenizerType:
    nom::AsChar
    + Copy
    + std::hash::Hash
    + std::cmp::Eq
    + std::convert::From<u8>
    + std::convert::From<char>
    + std::fmt::Debug
    + std::marker::Sync
{
}

impl<T> TokenizerType for T where
    T: nom::AsChar
        + Copy
        + std::hash::Hash
        + std::cmp::Eq
        + std::convert::From<u8>
        + std::convert::From<char>
        + std::fmt::Debug
        + std::marker::Sync
{
}

#[derive(Clone)]
pub struct Tokenizer<T: TokenizerType> {
    byte_encoder: HashMap<T, char>,
    byte_decoder: HashMap<char, T>,
    token_encoder: HashMap<String, u16>,
    token_decoder: HashMap<u16, String>,
    bpe_ranks: HashMap<BpePair, usize>,
}

impl<T: TokenizerType> Tokenizer<T> {
    pub fn new<P>(encoder_filename: P, vocab_filename: P) -> Tokenizer<T>
    where
        P: AsRef<Path>,
    {
        let token_encoder = create_bpe_token_encoder(encoder_filename).unwrap();
        let byte_encoder = create_bpe_char_encoder::<T>();

        Tokenizer::<T> {
            byte_encoder: byte_encoder.clone(),
            byte_decoder: create_bpe_char_decoder::<T>(byte_encoder),
            token_encoder: token_encoder.clone(),
            token_decoder: create_bpe_token_decoder(token_encoder).unwrap(),
            bpe_ranks: create_bpe_ranks(vocab_filename).unwrap(),
        }
    }

    pub fn tokenize<S>(&self, text: S) -> Vec<u16>
    where
        S: Into<String>,
    {
        //TODO: Handle unmatched
        let text = text.into();
        let (_unmatched, pat_tokens) = pat::pat(&text).unwrap();

        pat_tokens
            .into_par_iter()
            .flat_map(|token| {
                self.bpe_new(
                //self.bpe(
                    token
                        .chars()
                        .map(|c| self.byte_encoder[&c.into()])
                        .collect::<String>()
                )
                .split(" ")
                .map(|new_token| {
                    let encoded_token = self.token_encoder[new_token];
                    encoded_token
                })
                .collect::<Vec<_>>()
            })
            .collect()
    }

    pub fn detokenize(&self, tokens: Vec<u16>) -> String {
        tokens
            .into_iter()
            .map(|token| self.token_decoder[&token].clone())
            .collect::<Vec<_>>()
            .join("")
            .chars()
            .map(|c| self.byte_decoder.get(&c).unwrap().as_char())
            .collect()
    }

    //TODO Rename
    fn bpe<S>(&self, token: S) -> String
    where
        S: Into<String>,
    {
        //TODO Cache
        let token = token.into();

        let mut word = bpe_word_from_string(&token);

        loop {
            let pairs = generate_consecutive_pairs(&word);
            let Some(bigram) = pairs
                .into_iter()
                .min_by_key(|pair| self.bpe_ranks.get(pair)
                    .unwrap_or(&std::usize::MAX)
            ) else {
                break;
            };
            if !self.bpe_ranks.contains_key(&bigram) {
                break;
            }

            let (first, second) = bigram;
            let mut next_word: BpeWord = Vec::new();

            let mut i = 0;
            while i < word.len() {
                if let Some(mut j) = word
                    .clone()
                    .into_iter()
                    .skip(i)
                    .position(|sym| sym == first)
                {
                    j += i; //(adjust for skip(i)
                    let slice = word[i..j].to_vec();
                    next_word.extend(slice);
                    i = j
                } else {
                    let slice = word[i..].to_vec();
                    next_word.extend(slice);
                    break;
                }

                //TODO Not sure why we're comparing first, as it should be guaranteed
                if i < word.len() - 1 && word[i] == first && word[i + 1] == second {
                    let combined = first.clone() + &second;
                    next_word.push(combined);
                    i += 2;
                } else {
                    next_word.push(first.clone());
                    i += 1;
                }
                dbg!(&next_word);
            }

            word = next_word;
            if word.len() == 1 {
                break;
            }
        }

        word.join(" ")
    }

    fn bpe_new<S>(&self, token: S) -> String
    where
        S: Into<String>,
    {
        //TODO Cache
        let token = token.into();

        let mut word = bpe_word_from_string(&token);

        loop {
            let pairs = generate_consecutive_pairs(&word);
            let Some(bigram) = pairs
                .into_iter()
                .min_by_key(|pair| self.bpe_ranks.get(pair)
                    .unwrap_or(&std::usize::MAX)
            ) else {
                break;
            };
            if !self.bpe_ranks.contains_key(&bigram) {
                break;
            }

            let (first, second) = bigram;
            let mut next_word: BpeWord = Vec::new();

            let mut word_iter = word.into_iter().peekable();
            while let Some(symbol) = word_iter.next() {
                if symbol == first {
                    if word_iter.peek().is_some() {
                    let next_symbol = word_iter.peek().unwrap();
                        if *next_symbol == second {
                            let next_symbol = word_iter.next().unwrap();
                            next_word.push(symbol+&next_symbol);
                            continue
                        }
                    }
                }
                next_word.push(symbol);
            }
            /*
            let mut word_iter = word.windows(2).peekable();
            while let Some(pair) = word_iter.next() {
                let (a, b) = (pair[0].to_owned(), pair[1].to_owned());
                dbg!(&a, &b);
                if a == first && b == second {
                    next_word.push(a+&b);
                    word_iter.next();
                } else {
                    next_word.push(a);

                    if word_iter.peek().is_none() {
                        next_word.push(b);
                    }
                }

                dbg!(&next_word);
            }
            */

            word = next_word;
            if word.len() == 1 {
                break;
            }
        }

        word.join(" ")
    }
}
