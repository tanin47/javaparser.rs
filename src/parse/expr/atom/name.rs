use either::Either;
use parse::combinator::{any_keyword, identifier};
use parse::tpe::primitive;
use parse::tree::{Keyword, Name};
use parse::{ParseResult, Tokens};
use tokenize::span::Span;

pub fn parse(input: Tokens) -> ParseResult<Either<Keyword, Name>> {
    if let Ok((input, name)) = identifier(input) {
        Ok((input, Either::Right(Name { name })))
    } else if let Ok((input, name)) = any_keyword(input) {
        Ok((input, Either::Left(Keyword { name })))
    } else {
        Err(input)
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
