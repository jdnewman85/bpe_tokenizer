use nom::{
    IResult,
    branch::alt,
    bytes::complete::{tag, take_while1},
    combinator::opt, multi::many1, character::complete::char,
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
    let input_original = input;
    let (input, space) = opt(char(' '))(input)?;
    let (input, word) = take_while1(|c: char| c.is_alphabetic())(input)?;
    let length_of_result = word.len() + (space.map_or(0, |_| 1));
    //dbg!("sw");
    Ok((input, &input_original[..length_of_result]))
}

fn space_contraction(input: &str) -> IResult<&str, &str> {
    let input_original = input;
    let (input, space) = opt(char(' '))(input)?;
    let (input, cont) = contraction(input)?;
    let length_of_result = cont.len() + (space.map_or(0, |_| 1));
    //dbg!("sc");
    Ok((input, &input_original[..length_of_result]))
}

fn space_unicode_word(input: &str) -> IResult<&str, &str> {
    let input_original = input;
    let (input, space) = opt(char(' '))(input)?;
    let (input, word) = nom_unicode::complete::alpha1(input)?;
    let length_of_result = word.len() + (space.map_or(0, |_| 1));
    //dbg!("suw");
    //dbg!(input, word, space);
    Ok((input, &input_original[..length_of_result]))
}

fn space_unicode_number(input: &str) -> IResult<&str, &str> {
    let input_original = input;
    let (input, space) = opt(char(' '))(input)?;
    let (input, word) = nom_unicode::complete::digit1(input)?;
    let length_of_result = word.len() + (space.map_or(0, |_| 1));
    //dbg!("sun");
    Ok((input, &input_original[..length_of_result]))
}

fn is_non_unicode_space_alphanumeric(chr: char) -> bool {
    !(nom_unicode::is_alphanumeric(chr) | nom_unicode::is_whitespace(chr))
}

fn space_non_unicode_alphanumeric(input: &str) -> IResult<&str, &str> {
    let input_original = input;
    let (input, space) = opt(char(' '))(input)?;
    let (input, word) = take_while1(is_non_unicode_space_alphanumeric)(input)?;
    let length_of_result = word.len() + (space.map_or(0, |_| 1));
    //dbg!("snua");
    Ok((input, &input_original[..length_of_result]))
}

fn whitespace_then_non(input: &str) -> IResult<&str, &str> {
    take_while1(nom_unicode::is_whitespace)(input)
}

fn pat(input: &str) -> IResult<&str, Vec<&str>> {
    many1(
        alt((
            space_contraction,
            space_unicode_word,
            space_unicode_number,
            space_non_unicode_alphanumeric,
            nom_unicode::complete::space1,
        ))
    )(input)
}


fn main() {
    //println!("{:?}", pat("This is a test! y'all's allright?").unwrap());
    dbg!(pat("This is a test! y'all's allright?").unwrap());
}
