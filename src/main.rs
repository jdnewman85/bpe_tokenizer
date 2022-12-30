use std::{fs::File, path::Path};
use std::io::{BufRead, Read, BufReader};
use std::collections::{HashSet, HashMap};

use nom::{
    branch::alt,
    bytes::complete::{tag, take_while1},
    character::complete::char,
    combinator::{opt, recognize},
    error::ParseError,
    multi::many1,
    sequence::pair,
    AsChar, IResult, InputIter,
};
use nom_unicode;

fn contraction(input: &str) -> IResult<&str, &str> {
    alt((
        tag("'s"),
        tag("'t"),
        tag("'re"),
        tag("'ve"),
        tag("'m"),
        tag("'ll"),
        tag("'d"),
    ))(input)
}

fn opt_preceding_space<I, O, E: ParseError<I>, F>(f: F) -> impl FnMut(I) -> IResult<I, I, E>
where
    I: Clone
        + nom::InputIter
        + nom::Offset
        + nom::Slice<std::ops::RangeFrom<usize>>
        + nom::Slice<std::ops::RangeTo<usize>>,
    F: nom::Parser<I, O, E>, <I as InputIter>::Item: AsChar,
{
    recognize(pair(opt(char(' ')), f))
}

fn opt_space_contraction(input: &str) -> IResult<&str, &str> {
    opt_preceding_space(contraction)(input)
}

fn opt_space_unicode_alpha1(input: &str) -> IResult<&str, &str> {
    opt_preceding_space(nom_unicode::complete::alpha1)(input)
}

fn opt_space_unicode_digit1(input: &str) -> IResult<&str, &str> {
    opt_preceding_space(nom_unicode::complete::digit1)(input)
}

fn is_non_unicode_space_alphanumeric(chr: char) -> bool {
    !(nom_unicode::is_alphanumeric(chr) | nom_unicode::is_whitespace(chr))
}

fn opt_space_non_unicode_alphanumeric1(input: &str) -> IResult<&str, &str> {
    opt_preceding_space(take_while1(is_non_unicode_space_alphanumeric))(input)
}

fn pat(input: &str) -> IResult<&str, Vec<&str>> {
//TODO: Should this slit on and consume newlines? I don't think so?
    many1(alt((
        opt_space_contraction,
        opt_space_unicode_alpha1,
        opt_space_unicode_digit1,
        opt_space_non_unicode_alphanumeric1,
        /* Original regex has spaces before a non-space here, but that seems unneeded as it'd be
        * caught by the space1, then one of the above on next match itteration? */
        nom_unicode::complete::space1,
    )))(input)
}

fn is_valid_bpe_char<T>(i: T) -> bool
where T: nom::AsChar + Copy //TODO Or nom_unicode::IsChar?
{
    match i.as_char() {
        '!'..='~' |
        '¡'..='¬' |
        '®'..='ÿ' => true,
        _ => false,
    }
}

// Replaces bytes_to_unicode
fn create_bpe_char_encoder<T>() -> HashMap<T, char>
where T: nom::AsChar + Copy + std::hash::Hash + std::cmp::Eq + std::convert::From<u8> + std::fmt::Debug //TODO Or nom_unicode::IsChar?
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
            },
        };

        map.insert(i.into(), cs);
    }

//    dbg!(&map);
    return map
}

fn create_bpe_char_decoder<T>() -> HashMap<char, T>
where T: nom::AsChar + Copy + std::hash::Hash + std::cmp::Eq + std::convert::From<u8> + std::fmt::Debug //TODO Or nom_unicode::IsChar?
{
    //TODO Determine if this is efficient
    let map = create_bpe_char_encoder::<T>();
    let decoder = map.into_iter().map(|(k, v)| (v, k)).collect();

//    dbg!(&decoder);
    return decoder
}

type BpeTokenEncoder = HashMap<String, u16>;
fn create_bpe_token_encoder<T>(filename: T) -> Result<BpeTokenEncoder, std::io::Error>
where T: AsRef<Path>
{
    //TODO: Make more efficient
    let mut file = File::open(filename)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    let encoder: BpeTokenEncoder = serde_json::from_str(&contents)?;

    Ok(encoder)
}

type BpeTokenDecoder = HashMap<u16, String>;
fn create_bpe_token_decoder<T>(filename: T) -> Result<BpeTokenDecoder, std::io::Error>
where T: AsRef<Path>
{
    //TODO: Make more efficient
    let encoder = create_bpe_token_encoder::<T>(filename)?;
    let decoder: BpeTokenDecoder = encoder.into_iter().map(|(k, v)| (v, k)).collect();

    Ok(decoder)
}

type BpePair = (String, String);
type BpeRanks = Vec<BpePair>;
fn create_bpe_ranks<T>(filename: T) -> Result<BpeRanks, std::io::Error>
where T: AsRef<Path>
{
    //TODO: Make more efficient
    let file = File::open(filename)?;
    let reader = BufReader::new(file);

    //First line is version comment, hence skip(1)
    let num_comment_lines = 1;
    let ranks: BpeRanks = reader.lines().skip(num_comment_lines).map(|line| {
        let line = line.unwrap();
        let mut split = line.split_whitespace();
        (
            split.next().unwrap().to_string(),
            split.next().unwrap().to_string(),
        )
    }).collect();

    Ok(ranks)
}

type BpeWord = Vec<String>;
fn generate_consecutive_pairs(word: &BpeWord) -> HashSet<BpePair> {
    word.windows(2).map(|w| (w[0].to_owned(), w[1].to_owned())).collect()
}

fn bpe_word_from_string<T>(s: T) -> BpeWord
where T: Into<String>,
{
    s.into().chars().map(|c| c.to_string()).collect()
}

//TODO Rename
fn bpe(token: String) -> String {
    //TODO Cache


    let mut word = bpe_word_from_string(&token);
    let mut pairs = generate_consecutive_pairs(&word);

    if pairs.is_empty() {
        return token;
    }

    //TODO Factor this out!!
    let bpe_ranks = create_bpe_ranks("vocab.bpe").unwrap();
    let bpe_ranks = bpe_ranks.into_iter().enumerate().fold(HashMap::new(), |mut map, bpe_rank| {
        map.insert(bpe_rank.1, bpe_rank.0);
        map
    });

    loop {
        //min pair of all of
        //For each in pairs
        //get from bpe_ranks, or inf
        let Some(bigram) = pairs.clone().into_iter().min_by_key(|pair| bpe_ranks.get(pair).unwrap_or(&std::usize::MAX)) else {
            break;
        };

        let (first, second) = bigram;
        let mut next_word: BpeWord = Vec::new();

        let mut i = 0;
        while i < word.len() {
            if let Some(j) = word.clone().into_iter().skip(i).position(|sym| sym==first) {
                let slice = word[i..j].to_vec();
                next_word.extend(slice);
                i = j
            } else {
                let slice = word[i..].to_vec();
                next_word.extend(slice);
                break;
            }

            if i < word.len()-1 && word[i+1] == second {
                let combined = first.clone()+&second;
                next_word.push(combined);
                i += 2;
            } else {
                next_word.push(first.clone());
                i += 1;
            }
        }

        word = next_word;
        if word.len() == 1 {
            break;
        }

        pairs = generate_consecutive_pairs(&word);
    }

    let word = word.join(" ");
    word
}

fn main() {
    //println!("{:?}", pat("This is a test! y'all's allright?").unwrap());
//    dbg!(pat("This is a test! y'all's allright?\nDo newlines work?!%? 1535").unwrap());
//
//    let bpe_char_encoder = create_bpe_char_encoder::<u8>();
//    dbg!(bpe_char_encoder[&('A' as u8)]);
//
//    let bpe_char_encoder = create_bpe_char_encoder::<char>();
//    let bpe_char_decoder = create_bpe_char_decoder::<char>();
//    let space_encoded = bpe_char_encoder[&' '];
//    dbg!(space_encoded);
//    dbg!(bpe_char_decoder[&space_encoded]);
//
//    let bpe_token_encoder = create_bpe_token_encoder("encoder.json").unwrap();
//    dbg!(&bpe_token_encoder);
//    let bpe_token_decoder = create_bpe_token_decoder("encoder.json").unwrap();
//    dbg!(&bpe_token_decoder);
//
//    let bpe_ranks = create_bpe_ranks("vocab.bpe");
//    dbg!(&bpe_ranks);

    /*
    let test_pairs = generate_consecutive_pairs(&bpe_word_from_string("abcdefg"));
    dbg!(test_pairs);

    let test_pairs = generate_consecutive_pairs(&vec![
        "This".to_string(),
        "is".to_string(),
        "a".to_string(),
        "test".to_string(),
    ]);
    dbg!(test_pairs);
    */

    //encode
    let (_unmatched, pat_tokens) = pat("This is a test! y'all's allright?\nDo newlines work?!%? 1535").unwrap();

    let bpe_char_encoder = create_bpe_char_encoder::<char>();
    let bpe_token_encoder = create_bpe_token_encoder("encoder.json").unwrap();

    let mut bpe_tokens:Vec<u16> = Vec::new();

    for token in pat_tokens {
        let prepared_token:String = token.chars().map(|c| bpe_char_encoder[&c]).collect();
        let bpe_results = bpe(prepared_token);
        let new_bpe_tokens:Vec<u16> = bpe_results.split(" ").map(|new_token| {
            dbg!(new_token);
            let encoded_token = bpe_token_encoder[new_token];
//            dbg!(encoded_token);
            encoded_token
        }).collect();
        bpe_tokens.extend(new_bpe_tokens);
    }

    dbg!(bpe_tokens);

}
