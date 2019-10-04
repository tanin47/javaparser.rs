use parse::tree::{Expr, LiteralString};
use parse::{ParseResult, Tokens};
use tokenize::token::Token;

pub fn parse<'def, 'r>(input: Tokens<'def, 'r>) -> ParseResult<'def, 'r, Expr<'def>> {
    if let Token::String(value) = input[0] {
        Ok((&input[1..], Expr::String(LiteralString { value })))
    } else {
        Err(input)
    }
}

#[cfg(test)]
mod tests {
    use super::parse;
    use parse::tree::{Expr, LiteralString};
    use parse::Tokens;
    use test_common::{generate_tokens, span};

    #[test]
    fn test_string() {
        assert_eq!(
            parse(&generate_tokens(
                r#"
"abc"
            "#
            )),
            Ok((
                &[] as Tokens,
                Expr::String(LiteralString {
                    value: span(1, 1, "\"abc\"")
                })
            ))
        );
    }
}
