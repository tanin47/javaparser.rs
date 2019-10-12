use parse::combinator::symbol2;
use parse::expr::{precedence_3, precedence_4};
use parse::id_gen::IdGen;
use parse::tree::{BinaryOperation, Expr};
use parse::{ParseResult, Tokens};

pub fn parse<'def, 'r>(
    input: Tokens<'def, 'r>,
    id_gen: &mut IdGen,
) -> ParseResult<'def, 'r, Expr<'def>> {
    let (input, left) = precedence_4::parse(input, id_gen)?;
    precedence_3::parse_tail(left, input, id_gen)
}

pub fn parse_tail<'def, 'r>(
    left: Expr<'def>,
    input: Tokens<'def, 'r>,
    id_gen: &mut IdGen,
) -> ParseResult<'def, 'r, Expr<'def>> {
    if let Ok((input, operator)) = symbol2('|', '|')(input) {
        let (input, right) = precedence_4::parse(input, id_gen)?;

        let expr = Expr::BinaryOperation(BinaryOperation {
            left: Box::new(left),
            operator,
            right: Box::new(right),
        });

        precedence_3::parse_tail(expr, input, id_gen)
    } else {
        Ok((input, left))
    }
}

//#[cfg(test)]
//mod tests {
//    use test_common::{generate_tokens, span};
//
//    use super::parse;
//    use parse::tree::{BinaryOperation, Boolean, Expr, FieldAccess, Name};
//    use parse::Tokens;
//    use std::cell::RefCell;
//
//    #[test]
//    fn test_precedence() {
//        assert_eq!(
//            parse(&generate_tokens(
//                r#"
//true || false && t.a || false
//            "#
//            )),
//            Ok((
//                &[] as Tokens,
//                Expr::BinaryOperation(BinaryOperation {
//                    left: Box::new(Expr::BinaryOperation(BinaryOperation {
//                        left: Box::new(Expr::Boolean(Boolean {
//                            value: span(1, 1, "true")
//                        })),
//                        operator: span(1, 6, "||"),
//                        right: Box::new(Expr::BinaryOperation(BinaryOperation {
//                            left: Box::new(Expr::Boolean(Boolean {
//                                value: span(1, 9, "false")
//                            })),
//                            operator: span(1, 15, "&&"),
//                            right: Box::new(Expr::FieldAccess(FieldAccess {
//                                expr: Box::new(Expr::Name(Name {
//                                    name: span(1, 18, "t")
//                                })),
//                                field: Name {
//                                    name: span(1, 20, "a")
//                                },
//                                tpe_opt: RefCell::new(None)
//                            }))
//                        })),
//                    })),
//                    operator: span(1, 22, "||"),
//                    right: Box::new(Expr::Boolean(Boolean {
//                        value: span(1, 25, "false")
//                    }))
//                })
//            ))
//        );
//    }
//}
