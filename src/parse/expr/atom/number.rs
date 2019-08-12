use parse::tree::{Expr, Hex, Int, Long};
use parse::{ParseResult, Tokens};
use tokenize::token::Token;

pub fn parse(input: Tokens) -> ParseResult<Expr> {
    if let Token::Int(value) = input[0] {
        Ok((&input[1..], Expr::Int(Int { value })))
    } else if Token::Hex(value) = input[0] {
        Ok((&input[1..], Expr::Hex(Hex { value })))
    } else if Token::Long(value) = input[0] {
        Ok((&input[1..], Expr::Long(Long { value })))
    } else {
        Err(input)
    }
}

#[cfg(test)]
mod tests {
    use super::parse;
    use parse::tree::{Expr, Hex};
    use test_common::{code, span};

    #[test]
    fn test_int() {
        assert_eq!(
            parse(&code(
                r#"
0xab1cdef123
            "#
            )),
            Ok((
                span(1, 13, ""),
                Expr::Hex(Hex {
                    value: span(1, 1, "0xab1cdef123")
                })
            ))
        );
    }
}
