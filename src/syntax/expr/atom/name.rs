use either::Either;
use nom::branch::alt;
use nom::bytes::complete::{is_a, tag, take_while};
use nom::character::complete::{alpha1, alphanumeric0};
use nom::character::{is_alphanumeric, is_digit};
use nom::sequence::tuple;
use nom::{Compare, IResult, InputLength, InputTake, InputTakeAtPosition};
use syntax::tpe::primitive;
use syntax::tree::{Expr, Int, Keyword, LiteralString, MethodCall, Name, Span};
use syntax::{comment, expr};

pub fn identifier_tail(input: Span) -> IResult<Span, Span> {
    input.split_at_position_complete(|c| !is_alphanumeric(c as u8) && c != '_')
}

pub fn identifier(input: Span) -> IResult<Span, Span> {
    let (input, _) = comment::parse(input)?;
    let (_, (first, body)) = tuple((alt((alpha1, is_a("_"))), identifier_tail))(input)?;

    Ok(input.take_split(first.input_len() + body.input_len()))
}

pub fn is_reserved(input: Span) -> bool {
    input.fragment == "instanceof"
        || input.fragment == "true"
        || input.fragment == "false"
        || input.fragment == "new"
        || input.fragment == "null"
        || input.fragment == "class"
        || primitive::valid(input.fragment)
}

pub fn parse(input: Span) -> IResult<Span, Either<Keyword, Name>> {
    let (input, name) = identifier(input)?;

    if is_reserved(name) {
        Ok((input, Either::Left(Keyword { name })))
    } else {
        Ok((input, Either::Right(Name { name })))
    }
}

#[cfg(test)]
mod tests {
    use super::parse;
    use either::Either;
    use syntax::tree::{Expr, Int, LiteralString, Method, MethodCall, Name, ReturnStmt};
    use test_common::{code, span};

    #[test]
    fn test_bare() {
        assert_eq!(
            parse(code(
                r#"
name_something
            "#
                .trim()
            )),
            Ok((
                span(1, 15, ""),
                Either::Right(Name {
                    name: span(1, 1, "name_something"),
                })
            ))
        );
    }
}
