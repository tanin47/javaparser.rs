use parse::combinator::{any_keyword, identifier, many0, separated_list};
use parse::def::annotateds;
use parse::id_gen::IdGen;
use parse::tree::{Keyword, Modifier};
use parse::{ParseResult, Tokens};
use tokenize::span::Span;

pub fn parse<'def, 'r>(
    input: Tokens<'def, 'r>,
    id_gen: &mut IdGen,
) -> ParseResult<'def, 'r, Vec<Modifier<'def>>> {
    many0(|i| parse_single(i, id_gen))(input)
}

fn parse_single<'def, 'r>(
    input: Tokens<'def, 'r>,
    id_gen: &mut IdGen,
) -> ParseResult<'def, 'r, Modifier<'def>> {
    if let Ok((input, annotated)) = annotateds::parse_annotated(input, id_gen) {
        Ok((input, Modifier::Annotated(annotated)))
    } else if let Ok((input, keyword)) = keyword(input) {
        Ok((input, Modifier::Keyword(Keyword { name: keyword })))
    } else {
        Err(input)
    }
}

fn keyword<'def, 'r>(original: Tokens<'def, 'r>) -> ParseResult<'def, 'r, Span<'def>> {
    let (input, keyword) = any_keyword(original)?;

    match keyword.fragment {
        "abstract" | "default" | "final" | "native" | "private" | "protected" | "public"
        | "static" | "strictfp" | "synchronized" | "transient" | "volatile" => Ok((input, keyword)),
        _ => Err(original),
    }
}
