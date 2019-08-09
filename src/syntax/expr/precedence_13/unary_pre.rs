use nom::branch::alt;
use nom::IResult;

use syntax::expr::precedence_14;
use syntax::tree::{BinaryOperation, Expr, Span, UnaryOperation};
use syntax::{expr, tag};

pub fn parse(input: Span) -> IResult<Span, Expr> {
    let (input, operator) = alt((tag("++"), tag("--")))(input)?;
    let (input, expr) = precedence_14::parse(input)?;

    Ok((
        input,
        Expr::UnaryOperation(UnaryOperation {
            expr: Box::new(expr),
            operator,
            is_post: false,
        }),
    ))
}

#[cfg(test)]
mod tests {
    use syntax::tree::{
        ArrayAccess, ClassType, Expr, Int, LiteralString, Method, MethodCall, Name, ReturnStmt,
        TypeArg, UnaryOperation,
    };
    use test_common::{code, span};

    use super::parse;

    #[test]
    fn test_increment() {
        assert_eq!(
            parse(code(
                r#"
++abc
            "#
                .trim()
            )),
            Ok((
                span(1, 6, ""),
                Expr::UnaryOperation(UnaryOperation {
                    expr: Box::new(Expr::Name(Name {
                        name: span(1, 3, "abc")
                    })),
                    operator: span(1, 1, "++"),
                    is_post: false
                })
            ))
        );
    }

    #[test]
    fn test_decrement() {
        assert_eq!(
            parse(code(
                r#"
--abc
            "#
                .trim()
            )),
            Ok((
                span(1, 6, ""),
                Expr::UnaryOperation(UnaryOperation {
                    expr: Box::new(Expr::Name(Name {
                        name: span(1, 3, "abc")
                    })),
                    operator: span(1, 1, "--"),
                    is_post: false
                })
            ))
        );
    }
}
