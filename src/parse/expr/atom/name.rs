use either::Either;
use parse::combinator::identifier;
use parse::tpe::primitive;
use parse::tree::{Keyword, Name};
use parse::{ParseResult, Tokens};
use tokenize::span::Span;

pub fn is_reserved(input: Span) -> bool {
    input.fragment == "instanceof"
        || input.fragment == "true"
        || input.fragment == "false"
        || input.fragment == "new"
        || input.fragment == "null"
        || input.fragment == "class"
        || primitive::valid(input.fragment)
}

pub fn parse(input: Tokens) -> ParseResult<Either<Keyword, Name>> {
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
    use parse::tree::Name;
    use parse::Tokens;
    use test_common::{code, span};

    #[test]
    fn test_bare() {
        assert_eq!(
            parse(&code(
                r#"
name_something
            "#
            )),
            Ok((
                &[] as Tokens,
                Either::Right(Name {
                    name: span(1, 1, "name_something"),
                })
            ))
        );
    }
}
