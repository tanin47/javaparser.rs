use parse::combinator::symbol;
use parse::tree::{Char, Expr};
use parse::{ParseResult, Tokens};
use tokenize::token::Token;

pub fn parse(input: Tokens) -> ParseResult<Expr> {
    if let Token::Char(value) = input[0] {
        Ok((&input[1..], Expr::Char(Char { value })))
    } else {
        Err(input)
    }
}

#[cfg(test)]
mod tests {
    use super::parse;
    use parse::tree::{Char, Expr};
    use parse::Tokens;
    use test_common::{generate_tokens, span};

    #[test]
    fn test_string() {
        assert_eq!(
            parse(&generate_tokens(
                r#"
'a'
            "#
            )),
            Ok((
                &[] as Tokens,
                Expr::Char(Char {
                    value: span(1, 1, "'a'")
                })
            ))
        );
    }
}
