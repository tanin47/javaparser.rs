use nom::character::complete::multispace0;
use nom::character::is_space;
use nom::IResult;

use syntax::tree::{Class, Method, Statement};
use syntax::tree::{ReturnStmt, Span};
use syntax::{comment, expr, tag};

pub fn parse(input: Span) -> IResult<Span, Statement> {
    let (input, _) = tag("return")(input)?;

    let (input, expr_opt) = match tag(";")(input) as IResult<Span, Span> {
        Ok((input, _)) => (input, None),
        Err(_) => {
            let (input, expr) = expr::parse(input)?;
            let (input, _) = tag(";")(input)?;
            (input, Some(expr))
        }
    };

    Ok((input, Statement::Return(ReturnStmt { expr_opt })))
}

#[cfg(test)]
mod tests {
    use super::parse;
    use syntax::tree::{Expr, LiteralString, Method, ReturnStmt, Statement};
    use test_common::{code, span};

    #[test]
    fn test_return_void() {
        assert_eq!(
            parse(code(
                r#"
return;
            "#
                .trim()
            )),
            Ok((
                span(1, 8, ""),
                Statement::Return(ReturnStmt { expr_opt: None })
            ))
        );
    }

    #[test]
    fn test_return_string() {
        assert_eq!(
            parse(code(
                r#"
return "test";
            "#
                .trim()
            )),
            Ok((
                span(1, 15, ""),
                Statement::Return(ReturnStmt {
                    expr_opt: Some(Expr::String(LiteralString {
                        value: span(1, 9, "test")
                    }))
                })
            ))
        );
    }
}
