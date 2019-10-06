use parse::combinator::{get_and_not_followed_by, symbol, symbol2, symbol3, symbol4};
use parse::expr::{precedence_1, precedence_2};
use parse::tree::{Assigned, Assignment, Expr};
use parse::{ParseResult, Tokens};
use tokenize::span::Span;

fn op<'def, 'r>(input: Tokens<'def, 'r>) -> ParseResult<'def, 'r, Span<'def>> {
    if let Ok(ok) = get_and_not_followed_by(symbol('='), symbol('='))(input) {
        Ok(ok)
    } else if let Ok(ok) = symbol2('+', '=')(input) {
        Ok(ok)
    } else if let Ok(ok) = symbol2('-', '=')(input) {
        Ok(ok)
    } else if let Ok(ok) = symbol2('*', '=')(input) {
        Ok(ok)
    } else if let Ok(ok) = symbol2('/', '=')(input) {
        Ok(ok)
    } else if let Ok(ok) = symbol2('%', '=')(input) {
        Ok(ok)
    } else if let Ok(ok) = symbol2('|', '=')(input) {
        Ok(ok)
    } else if let Ok(ok) = symbol2('&', '=')(input) {
        Ok(ok)
    } else if let Ok(ok) = symbol2('^', '=')(input) {
        Ok(ok)
    } else if let Ok(ok) = symbol3('<', '<', '=')(input) {
        Ok(ok)
    } else if let Ok(ok) = symbol3('>', '>', '=')(input) {
        Ok(ok)
    } else if let Ok(ok) = symbol4('>', '>', '>', '=')(input) {
        Ok(ok)
    } else {
        Err(input)
    }
}

pub fn parse_tail<'def, 'r>(
    left: Expr<'def>,
    input: Tokens<'def, 'r>,
) -> ParseResult<'def, 'r, Expr<'def>> {
    let (input, operator) = match op(input) {
        Ok(ok) => ok,
        _ => return precedence_2::parse_tail(left, input),
    };

    let assigned = match left {
        Expr::FieldAccess(field) => Assigned::Field(field),
        Expr::ArrayAccess(arr) => Assigned::ArrayAccess(arr),
        Expr::Name(name) => Assigned::Name(name),
        _ => return Err(input),
    };
    let (input, expr) = precedence_1::parse(input)?;

    Ok((
        input,
        Expr::Assignment(Assignment {
            assigned: Box::new(assigned),
            operator,
            expr: Box::new(expr),
        }),
    ))
}

pub fn parse<'def, 'r>(input: Tokens<'def, 'r>) -> ParseResult<'def, 'r, Expr<'def>> {
    let (input, left) = precedence_2::parse(input)?;
    parse_tail(left, input)
}

//#[cfg(test)]
//mod tests {
//    use test_common::{generate_tokens, span};
//
//    use super::parse;
//    use parse::tree::{
//        ArrayAccess, Assigned, Assignment, BinaryOperation, Expr, FieldAccess, Int, Name,
//    };
//    use parse::Tokens;
//    use std::cell::RefCell;
//
//    #[test]
//    fn test_and_assignment() {
//        assert_eq!(
//            parse(&generate_tokens(
//                r#"
//a <<= b
//            "#
//            )),
//            Ok((
//                &[] as Tokens,
//                Expr::Assignment(Assignment {
//                    assigned: Box::new(Assigned::Name(Name {
//                        name: span(1, 1, "a")
//                    })),
//                    operator: span(1, 3, "<<="),
//                    expr: Box::new(Expr::Name(Name {
//                        name: span(1, 7, "b")
//                    }))
//                })
//            ))
//        );
//    }
//
//    #[test]
//    fn test_longest_assignment() {
//        assert_eq!(
//            parse(&generate_tokens(
//                r#"
//a >>>= b
//            "#
//            )),
//            Ok((
//                &[] as Tokens,
//                Expr::Assignment(Assignment {
//                    assigned: Box::new(Assigned::Name(Name {
//                        name: span(1, 1, "a")
//                    })),
//                    operator: span(1, 3, ">>>="),
//                    expr: Box::new(Expr::Name(Name {
//                        name: span(1, 8, "b")
//                    }))
//                })
//            ))
//        );
//    }
//
//    #[test]
//    fn test_assignment() {
//        assert_eq!(
//            parse(&generate_tokens(
//                r#"
//a = b.a += c.d[0][1] *= 1 == 2
//            "#
//            )),
//            Ok((
//                &[] as Tokens,
//                Expr::Assignment(Assignment {
//                    assigned: Box::new(Assigned::Name(Name {
//                        name: span(1, 1, "a")
//                    })),
//                    operator: span(1, 3, "="),
//                    expr: Box::new(Expr::Assignment(Assignment {
//                        assigned: Box::new(Assigned::Field(FieldAccess {
//                            expr: Box::new(Expr::Name(Name {
//                                name: span(1, 5, "b")
//                            })),
//                            field: Name {
//                                name: span(1, 7, "a")
//                            },
//                            tpe_opt: RefCell::new(None)
//                        })),
//                        operator: span(1, 9, "+="),
//                        expr: Box::new(Expr::Assignment(Assignment {
//                            assigned: Box::new(Assigned::ArrayAccess(ArrayAccess {
//                                expr: Box::new(Expr::ArrayAccess(ArrayAccess {
//                                    expr: Box::new(Expr::FieldAccess(FieldAccess {
//                                        expr: Box::new(Expr::Name(Name {
//                                            name: span(1, 12, "c")
//                                        })),
//                                        field: Name {
//                                            name: span(1, 14, "d")
//                                        },
//                                        tpe_opt: RefCell::new(None)
//                                    })),
//                                    index: Box::new(Expr::Int(Int {
//                                        value: span(1, 16, "0")
//                                    }))
//                                })),
//                                index: Box::new(Expr::Int(Int {
//                                    value: span(1, 19, "1")
//                                }))
//                            })),
//                            operator: span(1, 22, "*="),
//                            expr: Box::new(Expr::BinaryOperation(BinaryOperation {
//                                left: Box::new(Expr::Int(Int {
//                                    value: span(1, 25, "1")
//                                })),
//                                operator: span(1, 27, "=="),
//                                right: Box::new(Expr::Int(Int {
//                                    value: span(1, 30, "2")
//                                }))
//                            }))
//                        }))
//                    }))
//                })
//            ))
//        );
//    }
//}
