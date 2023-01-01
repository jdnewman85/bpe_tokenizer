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
    F: nom::Parser<I, O, E>,
    <I as InputIter>::Item: AsChar,
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

pub fn pat(input: &str) -> IResult<&str, Vec<&str>> {
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
