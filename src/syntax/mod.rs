pub mod comment;
pub mod compilation_unit;
pub mod def;
pub mod expr;
pub mod imports;
pub mod statement;
pub mod tpe;
pub mod tree;

pub use self::compilation_unit::parse;
use nom::bytes::complete::{is_not, tag as nom_tag, take_while};
use nom::character::{is_alphanumeric, is_digit};
use nom::combinator::peek;
use nom::error::ErrorKind;
use nom::sequence::tuple;
use nom::IResult;
use syntax::tree::Span;

pub fn tag<'a>(s: &'a str) -> impl Fn(Span<'a>) -> IResult<Span<'a>, Span<'a>> {
    move |input: Span<'a>| {
        let (input, _) = comment::parse(input)?;
        nom_tag(s)(input)
    }
}

fn is_valid_word_char(c: char) -> bool {
    is_alphanumeric(c as u8) || is_digit(c as u8) || c == '_'
}

pub fn word<'a>(s: &'a str) -> impl Fn(Span<'a>) -> IResult<Span<'a>, Span<'a>> {
    move |original: Span<'a>| {
        let (input, span) = tag(s)(original)?;
        let (_, result) = take_while(is_valid_word_char)(input)?;

        if result.fragment == "" {
            Ok((input, span))
        } else {
            Err(nom::Err::Error((original, ErrorKind::Tag)))
        }
    }
}

pub fn tag_and_followed_by<'a, T>(
    s: &'a str,
    followed_by: T,
) -> impl Fn(Span<'a>) -> IResult<Span<'a>, Span<'a>>
where
    T: Fn(Span<'a>) -> IResult<Span<'a>, Span<'a>>,
{
    move |input: Span<'a>| {
        let (input, _) = comment::parse(input)?;
        let (input, tagged) = nom_tag(s)(input)?;
        let _ = followed_by(input)?;

        Ok((input, tagged))
    }
}
