use nom::{
    IResult,
    branch::alt,
    bytes::complete::{tag, take_while1},
    combinator::{opt, recognize}, multi::many1, character::complete::char, sequence::pair,
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

fn space_word(input: &str) -> IResult<&str, &str> {
    recognize(
        pair(
            opt(char(' ')),
            take_while1(|c: char| c.is_alphabetic()),
        )
    )(input)
}

fn space_word_alt(input: &str) -> IResult<&str, &str> {
    let input_original = input;
    let (input, space) = opt(char(' '))(input)?;
    let (input, word) = take_while1(|c: char| c.is_alphabetic())(input)?;
    let length_of_result = word.len() + (space.map_or(0, |_| 1));
    //dbg!("sw");
    Ok((input, &input_original[..length_of_result]))
}

fn space_contraction(input: &str) -> IResult<&str, &str> {
    recognize(
        pair(
            opt(char(' ')),
            contraction,
        )
    )(input)
}

fn space_unicode_alpha1(input: &str) -> IResult<&str, &str> {
    recognize(
        pair(
            opt(char(' ')),
            nom_unicode::complete::alpha1,
        )
    )(input)
}

fn space_unicode_digit1(input: &str) -> IResult<&str, &str> {
    recognize(
        pair(
            opt(char(' ')),
            nom_unicode::complete::digit1,
        )
    )(input)
}

fn is_non_unicode_space_alphanumeric(chr: char) -> bool {
    !(nom_unicode::is_alphanumeric(chr) | nom_unicode::is_whitespace(chr))
}

fn space_non_unicode_alphanumeric1(input: &str) -> IResult<&str, &str> {
    recognize(
        pair(
            opt(char(' ')),
            take_while1(is_non_unicode_space_alphanumeric)
        )
    )(input)
}

fn pat(input: &str) -> IResult<&str, Vec<&str>> {
    many1(
        alt((
            space_contraction,
            space_unicode_alpha1,
            space_unicode_digit1,
            space_non_unicode_alphanumeric1,
            nom_unicode::complete::space1,
        ))
    )(input)
}


fn main() {
    //println!("{:?}", pat("This is a test! y'all's allright?").unwrap());
    dbg!(pat("This is a test! y'all's allright?\nDo newlines work?!%? 1535").unwrap());
}
