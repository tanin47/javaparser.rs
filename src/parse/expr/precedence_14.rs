use nom::branch::alt;
use nom::IResult;
use syntax::expr::atom::{method_call, name};
use syntax::expr::precedence_15;
use syntax::tree::{BinaryOperation, Expr, Span, UnaryOperation};
use syntax::{comment, expr, tag};

pub fn parse(input: Tokens) -> ParseResult<Expr> {
    let (input, expr) = precedence_15::parse(input)?;

    let (input, operator) = match alt((tag("++"), tag("--")))(input) {
        Ok(ok) => ok,
        Err(_) => return Ok((input, expr)),
    };

    Ok((
        input,
        Expr::UnaryOperation(UnaryOperation {
            expr: Box::new(expr),
            operator,
            is_post: true,
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
abc++
            "#
                .trim()
            )),
            Ok((
                span(1, 6, ""),
                Expr::UnaryOperation(UnaryOperation {
                    expr: Box::new(Expr::Name(Name {
                        name: span(1, 1, "abc")
                    })),
                    operator: span(1, 4, "++"),
                    is_post: true
                })
            ))
        );
    }

    #[test]
    fn test_decrement() {
        assert_eq!(
            parse(code(
                r#"
abc--
            "#
                .trim()
            )),
            Ok((
                span(1, 6, ""),
                Expr::UnaryOperation(UnaryOperation {
                    expr: Box::new(Expr::Name(Name {
                        name: span(1, 1, "abc")
                    })),
                    operator: span(1, 4, "--"),
                    is_post: true
                })
            ))
        );
    }
}
