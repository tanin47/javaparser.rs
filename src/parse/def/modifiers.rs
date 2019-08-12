use parse::combinator::{identifier, many0, separated_list};
use parse::def::annotateds;
use parse::tree::{Keyword, Modifier};
use parse::{ParseResult, Tokens};
use tokenize::span::Span;

pub fn parse(input: Tokens) -> ParseResult<Vec<Modifier>> {
    many0(parse_single)(input)
}

fn parse_single(input: Tokens) -> ParseResult<Modifier> {
    if let Ok((input, annotated)) = annotateds::parse_annotated(input) {
        Ok((input, Modifier::Annotated(annotated)))
    } else if let Ok((input, keyword)) = keyword(input) {
        Ok((input, Modifier::Keyword(Keyword { name: keyword })))
    } else {
        Err(input)
    }
}

fn keyword(original: Tokens) -> ParseResult<Span> {
    let (input, keyword) = identifier(original)?;

    match keyword.fragment {
        "abstract" | "default" | "final" | "native" | "private" | "protected" | "public"
        | "static" | "strictfp" | "synchronized" | "transient" | "volatile" => Ok((input, keyword)),
        _ => Err(original),
    }
}
