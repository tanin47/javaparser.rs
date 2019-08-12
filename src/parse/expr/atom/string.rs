use parse::tree::{Expr, LiteralString};
use parse::{ParseResult, Tokens};
use tokenize::token::Token;

pub fn parse(input: Tokens) -> ParseResult<Expr> {
    if let Token::String(value) = input[0] {
        Ok((input, Expr::String(LiteralString { value })))
    } else {
        Err(input)
    }
}

#[cfg(test)]
mod tests {
    use super::parse;
    use parse::tree::{Expr, LiteralString};
    use parse::Tokens;
    use test_common::{code, span};

    #[test]
    fn test_string() {
        assert_eq!(
            parse(&code(
                r#"
"abc"
            "#
            )),
            Ok((
                &[] as Tokens,
                Expr::String(LiteralString {
                    value: span(1, 2, "abc")
                })
            ))
        );
    }
}
