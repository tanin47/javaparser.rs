use parse::combinator::{keyword, symbol};
use parse::tree::{Assert, Statement};
use parse::{expr, ParseResult, Tokens};

pub fn parse(input: Tokens) -> ParseResult<Statement> {
    let (input, _) = keyword("assert")(input)?;
    let (input, expr) = expr::parse(input)?;

    let (input, error_opt) = if let Ok((input, _)) = symbol(':')(input) {
        let (input, error) = expr::parse(input)?;
        (input, Some(error))
    } else {
        (input, None)
    };

    let (input, _) = symbol(';')(input)?;

    Ok((input, Statement::Assert(Assert { expr, error_opt })))
}

#[cfg(test)]
mod tests {
    use super::parse;
    use parse::tree::{Assert, Boolean, Expr, LiteralString, Statement};
    use parse::Tokens;
    use test_common::{code, span};

    #[test]
    fn test_bare() {
        assert_eq!(
            parse(&code(
                r#"
assert true;
            "#
            )),
            Ok((
                &[] as Tokens,
                Statement::Assert(Assert {
                    expr: Expr::Boolean(Boolean {
                        value: span(1, 8, "true")
                    }),
                    error_opt: None
                })
            ))
        );
    }

    #[test]
    fn test_error() {
        assert_eq!(
            parse(&code(
                r#"
assert true : "error";
            "#
            )),
            Ok((
                &[] as Tokens,
                Statement::Assert(Assert {
                    expr: Expr::Boolean(Boolean {
                        value: span(1, 8, "true")
                    }),
                    error_opt: Some(Expr::String(LiteralString {
                        value: span(1, 15, "\"error\"")
                    }))
                })
            ))
        );
    }
}
