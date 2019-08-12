use parse::expr::precedence_14;
use parse::tree::Expr;
use parse::{ParseResult, Tokens};

pub mod cast;
pub mod unary;
pub mod unary_pre;

pub fn parse(input: Tokens) -> ParseResult<Expr> {
    if let Ok(ok) = unary_pre::parse(input) {
        Ok(ok)
    } else if let Ok(ok) = unary::parse(input) {
        Ok(ok)
    } else if let Ok(ok) = cast::parse(input) {
        Ok(ok)
    } else {
        precedence_14::parse(input)
    }
}

#[cfg(test)]
mod tests {
    use test_common::{code, span};

    use super::parse;
    use parse::tree::{Cast, Expr, Name, PrimitiveType, Type, UnaryOperation};
    use parse::Tokens;

    #[test]
    fn test_multi() {
        assert_eq!(
            parse(&code(
                r#"
(int) +a
            "#
            )),
            Ok((
                &[] as Tokens,
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
