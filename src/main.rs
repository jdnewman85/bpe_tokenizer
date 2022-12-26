use nom::{
    IResult,
    branch::alt,
    bytes::complete::{tag, take_while1},
    combinator::{opt, recognize}, multi::many1, character::complete::char, sequence::pair, error::ParseError, InputIter, AsChar,
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
    I: Clone + nom::Slice<std::ops::RangeTo<usize>> + nom::Offset + nom::InputIter + nom::Slice<std::ops::RangeFrom<usize>>,
    F: nom::Parser<I, O, E>, <I as InputIter>::Item: AsChar
{
    recognize(
        pair(
            opt(char(' ')),
            f,
        )
    )
}

fn space_contraction(input: &str) -> IResult<&str, &str> {
    opt_preceding_space(
        contraction
    )(input)
}

fn space_unicode_alpha1(input: &str) -> IResult<&str, &str> {
    opt_preceding_space(
        nom_unicode::complete::alpha1,
    )(input)
}

fn space_unicode_digit1(input: &str) -> IResult<&str, &str> {
    opt_preceding_space(
        nom_unicode::complete::digit1,
    )(input)
}

fn is_non_unicode_space_alphanumeric(chr: char) -> bool {
    !(nom_unicode::is_alphanumeric(chr) | nom_unicode::is_whitespace(chr))
}

fn space_non_unicode_alphanumeric1(input: &str) -> IResult<&str, &str> {
    opt_preceding_space(
        take_while1(is_non_unicode_space_alphanumeric)
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
