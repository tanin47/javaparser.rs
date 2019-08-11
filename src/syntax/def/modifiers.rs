use nom::branch::alt;
use nom::combinator::peek;
use nom::error::ErrorKind;
use nom::multi::separated_list;
use nom::IResult;
use syntax::def::annotateds;
use syntax::tree::{Keyword, Modifier, Span};
use syntax::{comment, tag};

pub fn parse(input: Span) -> IResult<Span, Vec<Modifier>> {
    separated_list(comment::parse1, parse_single)(input)
}

fn parse_single(input: Span) -> IResult<Span, Modifier> {
    if let Ok((input, annotated)) = annotateds::parse_annotated(input) {
        Ok((input, Modifier::Annotated(annotated)))
    } else if let Ok((input, keyword)) = keyword(input) {
        Ok((input, Modifier::Keyword(Keyword { name: keyword })))
    } else {
        Err(nom::Err::Error((input, ErrorKind::Tag)))
    }
}

fn keyword(input: Span) -> IResult<Span, Span> {
    let (input, _) = comment::parse(input)?;
    let (input, keyword) = alt((
        tag("abstract"),
        tag("default"),
        tag("final"),
        tag("native"),
        tag("private"),
        tag("protected"),
        tag("public"),
        tag("static"),
        tag("strictfp"),
        tag("synchronized"),
        tag("transient"),
        tag("volatile"),
    ))(input)?;

    Ok((input, keyword))
}
