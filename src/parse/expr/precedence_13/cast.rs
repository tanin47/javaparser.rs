use parse::combinator::symbol;
use parse::expr::precedence_13;
use parse::tree::{Cast, Expr};
use parse::{tpe, ParseResult, Tokens};

pub fn parse(input: Tokens) -> ParseResult<Expr> {
    let (input, _) = symbol('(')(input)?;
    let (input, tpe) = tpe::parse(input)?;
    let (input, _) = symbol(')')(input)?;
    let (input, expr) = precedence_13::parse(input)?;

    Ok((
        input,
        Expr::Cast(Cast {
            tpe,
            expr: Box::new(expr),
        }),
    ))
}

#[cfg(test)]
mod tests {
    use test_common::{code, primitive, span};

    use super::parse;
    use parse::tree::{Cast, ClassType, Expr, Name, Type};

    #[test]
    fn test_multi() {
        assert_eq!(
            parse(&code(
                r#"
(boolean)(Int)t
            "#
            )),
            Ok((
                span(1, 16, ""),
                Expr::Cast(Cast {
                    expr: Box::new(Expr::Cast(Cast {
                        expr: Box::new(Expr::Name(Name {
                            name: span(1, 15, "t")
                        })),
                        tpe: Type::Class(ClassType {
                            prefix_opt: None,
                            name: span(1, 11, "Int"),
                            type_args_opt: None
                        })
                    })),
                    tpe: primitive(1, 2, "boolean")
                })
            ))
        );
    }
}
