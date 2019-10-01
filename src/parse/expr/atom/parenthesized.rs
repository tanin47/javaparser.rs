use parse::combinator::symbol;
use parse::expr::atom::array_access;
use parse::tree::Expr;
use parse::{expr, ParseResult, Tokens};

pub fn parse(input: Tokens) -> ParseResult<Expr> {
    let (input, _) = symbol('(')(input)?;
    let (input, expr) = expr::parse(input)?;
    let (input, _) = symbol(')')(input)?;

    array_access::parse_tail(input, expr)
}

#[cfg(test)]
mod tests {
    use super::parse;
    use parse::tree::{ClassType, Expr, InstanceOf, Int, Name, Type};
    use parse::Tokens;
    use test_common::{code, span};

    #[test]
    fn test_instanceof() {
        assert_eq!(
            parse(&code(
                r#"
(a instanceof Class)
            "#
            )),
            Ok((
                &[] as Tokens,
                Expr::InstanceOf(InstanceOf {
                    expr: Box::new(Expr::Name(Name {
                        name: span(1, 2, "a")
                    })),
                    operator: span(1, 4, "instanceof"),
                    tpe: Type::Class(ClassType {
                        prefix_opt: None,
                        name: span(1, 15, "Class"),
                        type_args_opt: None,
                        def_opt: None
                    })
                })
            ))
        );
    }

    #[test]
    fn test() {
        assert_eq!(
            parse(&code(
                r#"
(123)
            "#
            )),
            Ok((
                &[] as Tokens,
                Expr::Int(Int {
                    value: span(1, 2, "123")
                }),
            ))
        );
    }

    #[test]
    fn test_multi() {
        assert_eq!(
            parse(&code(
                r#"
(((123)))
            "#
            )),
            Ok((
                &[] as Tokens,
                Expr::Int(Int {
                    value: span(1, 4, "123")
                }),
            ))
        );
    }
}
