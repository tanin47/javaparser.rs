use nom::branch::alt;
use nom::IResult;
use syntax::expr::precedence_14;
use syntax::tree::{Expr, Span};

pub mod cast;
pub mod unary;
pub mod unary_pre;

pub fn parse(input: Span) -> IResult<Span, Expr> {
    alt((
        unary_pre::parse,
        unary::parse,
        cast::parse,
        precedence_14::parse,
    ))(input)
}

#[cfg(test)]
mod tests {
    use test_common::{code, span};

    use super::parse;
    use syntax::tree::{Cast, Expr, Name, PrimitiveType, Type, UnaryOperation};

    #[test]
    fn test_multi() {
        assert_eq!(
            parse(code(
                r#"
(int) +a
            "#
                .trim()
            )),
            Ok((
                span(1, 9, ""),
                Expr::Cast(Cast {
                    tpe: Type::Primitive(PrimitiveType {
                        name: span(1, 2, "int")
                    }),
                    expr: Box::new(Expr::UnaryOperation(UnaryOperation {
                        expr: Box::new(Expr::Name(Name {
                            name: span(1, 8, "a")
                        })),
                        operator: span(1, 7, "+"),
                        is_post: false
                    })),
                })
            ))
        );
    }
}
