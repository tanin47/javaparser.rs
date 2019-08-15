use parse::combinator::{keyword, symbol};
use parse::tree::{ReturnStmt, Statement};
use parse::{expr, ParseResult, Tokens};

pub fn parse(input: Tokens) -> ParseResult<Statement> {
    let (input, _) = keyword("return")(input)?;

    let (input, expr_opt) = match symbol(';')(input) {
        Ok((input, _)) => (input, None),
        Err(_) => {
            let (input, expr) = expr::parse(input)?;
            let (input, _) = symbol(';')(input)?;
            (input, Some(expr))
        }
    };

    Ok((input, Statement::Return(ReturnStmt { expr_opt })))
}

#[cfg(test)]
mod tests {
    use super::parse;
    use parse::tree::{Expr, LiteralString, ReturnStmt, Statement};
    use parse::Tokens;
    use test_common::{code, span};

    #[test]
    fn test_return_void() {
        assert_eq!(
            parse(&code(
                r#"
return;
            "#
            )),
            Ok((
                &[] as Tokens,
                Statement::Return(ReturnStmt { expr_opt: None })
            ))
        );
    }

    #[test]
    fn test_return_string() {
        assert_eq!(
            parse(&code(
                r#"
return "test";
            "#
            )),
            Ok((
                &[] as Tokens,
                Statement::Return(ReturnStmt {
                    expr_opt: Some(Expr::String(LiteralString {
                        value: span(1, 8, "\"test\"")
                    }))
                })
            ))
        );
    }
}
