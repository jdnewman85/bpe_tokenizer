use std::collections::HashMap;

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

/* python
def bytes_to_unicode():
    """
    Returns list of utf-8 byte and a corresponding list of unicode strings.
    The reversible bpe codes work on unicode strings.
    This means you need a large # of unicode characters in your vocab if you want to avoid UNKs.
    When you're at something like a 10B token dataset you end up needing around 5K for decent coverage.
    This is a signficant percentage of your normal, say, 32K bpe vocab.
    To avoid that, we want lookup tables between utf-8 bytes and unicode strings.
    And avoids mapping to whitespace/control characters the bpe code barfs on.
    """
    bs = list(range(ord("!"), ord("~")+1))+list(range(ord("¡"), ord("¬")+1))+list(range(ord("®"), ord("ÿ")+1))
    cs = bs[:]
    n = 0
    for b in range(2**8):
        if b not in bs:
            bs.append(b)
            cs.append(2**8+n)
            n += 1
    cs = [chr(n) for n in cs]
    return dict(zip(bs, cs))
*/

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
    //TODO: Use create_bpe_char_encoder, and swap key/values?
    //TODO: OR Refactor common itterator?
    let mut map: HashMap<char, T> = HashMap::new();

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

        map.insert(cs, i.into());
    }

//    dbg!(&map);
    return map
}


fn main() {
    //println!("{:?}", pat("This is a test! y'all's allright?").unwrap());
    dbg!(pat("This is a test! y'all's allright?\nDo newlines work?!%? 1535").unwrap());

    let bpe_char_encoder = create_bpe_char_encoder::<u8>();
    dbg!(bpe_char_encoder[&('A' as u8)]);

    let bpe_char_encoder = create_bpe_char_encoder::<char>();
    dbg!(bpe_char_encoder[&' ']);
}
