use parse::expr::precedence_14;
use parse::id_gen::IdGen;
use parse::tree::Expr;
use parse::{ParseResult, Tokens};

pub mod cast;
pub mod unary;
pub mod unary_pre;

pub fn parse<'def, 'r>(
    input: Tokens<'def, 'r>,
    id_gen: &mut IdGen,
) -> ParseResult<'def, 'r, Expr<'def>> {
    if let Ok(ok) = unary_pre::parse(input, id_gen) {
        Ok(ok)
    } else if let Ok(ok) = unary::parse(input, id_gen) {
        Ok(ok)
    } else if let Ok(ok) = cast::parse(input, id_gen) {
        Ok(ok)
    } else {
        precedence_14::parse(input, id_gen)
    }
}

//#[cfg(test)]
//mod tests {
//    use test_common::{generate_tokens, span};
//
//    use super::parse;
//    use parse::tree::{Cast, Expr, Name, PrimitiveType, PrimitiveTypeType, Type, UnaryOperation};
//    use parse::Tokens;
//
//    #[test]
//    fn test_multi() {
//        assert_eq!(
//            parse(&generate_tokens(
//                r#"
//(int) +a
//            "#
//            )),
//            Ok((
//                &[] as Tokens,
//                Expr::Cast(Cast {
//                    tpes: vec![Type::Primitive(PrimitiveType {
//                        name: span(1, 2, "int"),
//                        tpe: PrimitiveTypeType::Int
//                    })],
//                    expr: Box::new(Expr::UnaryOperation(UnaryOperation {
//                        expr: Box::new(Expr::Name(Name {
//                            name: span(1, 8, "a")
//                        })),
//                        operator: span(1, 7, "+"),
//                        is_post: false
//                    })),
//                })
//            ))
//        );
//    }
//}
