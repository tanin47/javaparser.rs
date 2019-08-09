use nom::branch::alt;
use nom::IResult;
use syntax::expr::precedence_13;
use syntax::tag;
use syntax::tree::{Expr, Span, UnaryOperation};

pub fn parse(input: Span) -> IResult<Span, Expr> {
    let (input, operator) = alt((tag("+"), tag("-"), tag("!"), tag("~")))(input)?;

    let (input, expr) = precedence_13::parse(input)?;

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
    fn test_multi() {
        assert_eq!(
            parse(code(
                r#"
+-a
            "#
                .trim()
            )),
            Ok((
                span(1, 4, ""),
                Expr::UnaryOperation(UnaryOperation {
                    expr: Box::new(Expr::UnaryOperation(UnaryOperation {
                        expr: Box::new(Expr::Name(Name {
                            name: span(1, 3, "a")
                        })),
                        operator: span(1, 2, "-"),
                        is_post: false
                    })),
                    operator: span(1, 1, "+"),
                    is_post: false
                })
            ))
        );
    }
}
