use nom::character::is_digit;
use nom::IResult;
use syntax::expr::atom::array_access;
use syntax::tree::{Expr, Int, Span};
use syntax::{comment, expr, tag};

pub fn parse(input: Span) -> IResult<Span, Expr> {
    let (input, _) = tag("(")(input)?;
    let (input, expr) = expr::parse(input)?;
    let (input, _) = tag(")")(input)?;

    array_access::parse_tail(input, expr)
}

#[cfg(test)]
mod tests {
    use super::parse;
    use syntax::tree::{
        BinaryOperation, ClassType, Expr, InstanceOf, Int, Method, MethodCall, Name, ReturnStmt,
        Type,
    };
    use test_common::{code, span};

    #[test]
    fn test_instanceof() {
        assert_eq!(
            parse(code(
                r#"
(a instanceof Class)
            "#
                .trim()
            )),
            Ok((
                span(1, 21, ""),
                Expr::InstanceOf(InstanceOf {
                    expr: Box::new(Expr::Name(Name {
                        name: span(1, 2, "a")
                    })),
                    operator: span(1, 4, "instanceof"),
                    tpe: Type::Class(ClassType {
                        prefix_opt: None,
                        name: span(1, 15, "Class"),
                        type_args_opt: None
                    })
                })
            ))
        );
    }

    #[test]
    fn test() {
        assert_eq!(
            parse(code(
                r#"
(123)
            "#
                .trim()
            )),
            Ok((
                span(1, 6, ""),
                Expr::Int(Int {
                    value: span(1, 2, "123")
                }),
            ))
        );
    }

    #[test]
    fn test_multi() {
        assert_eq!(
            parse(code(
                r#"
(((123)))
            "#
                .trim()
            )),
            Ok((
                span(1, 10, ""),
                Expr::Int(Int {
                    value: span(1, 4, "123")
                }),
            ))
        );
    }
}
