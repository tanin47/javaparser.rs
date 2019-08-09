use nom::branch::alt;
use nom::IResult;
use syntax::expr::precedence_13;
use syntax::tree::{Cast, Expr, Span};
use syntax::{tag, tpe};

pub fn parse(input: Span) -> IResult<Span, Expr> {
    let (input, _) = tag("(")(input)?;
    let (input, tpe) = tpe::parse(input)?;
    let (input, _) = tag(")")(input)?;
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
    use syntax::tree::{
        ArrayAccess, Cast, ClassType, Expr, Int, LiteralString, Method, MethodCall, Name,
        ReturnStmt, Type, TypeArg, UnaryOperation,
    };
    use test_common::{code, primitive, span};

    use super::parse;

    #[test]
    fn test_multi() {
        assert_eq!(
            parse(code(
                r#"
(boolean)(Int)t
            "#
                .trim()
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
