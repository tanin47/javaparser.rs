use parse::tree::{Double, Expr, Float, Hex, Int, Long};
use parse::{ParseResult, Tokens};
use tokenize::token::Token;

pub fn parse<'def, 'r>(input: Tokens<'def, 'r>) -> ParseResult<'def, 'r, Expr<'def>> {
    if let Token::Int(value) = input[0] {
        Ok((&input[1..], Expr::Int(Int { value })))
    } else if let Token::Long(value) = input[0] {
        Ok((&input[1..], Expr::Long(Long { value })))
    } else if let Token::Double(value) = input[0] {
        Ok((&input[1..], Expr::Double(Double { value })))
    } else if let Token::Float(value) = input[0] {
        Ok((&input[1..], Expr::Float(Float { value })))
    } else {
        Err(input)
    }
}

#[cfg(test)]
mod tests {
    use super::parse;
    use parse::tree::{Expr, Int};
    use parse::Tokens;
    use test_common::{generate_tokens, span};

    #[test]
    fn test_int() {
        assert_eq!(
            parse(&generate_tokens(
                r#"
0xab1cdef123
            "#
            )),
            Ok((
                &[] as Tokens,
                Expr::Int(Int {
                    value: span(1, 1, "0xab1cdef123")
                })
            ))
        );
    }
}
