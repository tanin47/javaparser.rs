use nom::branch::alt;
use nom::bytes::complete::is_not;
use nom::combinator::{map, opt, peek};
use nom::error::ErrorKind;
use nom::sequence::tuple;
use nom::{FindSubstring, IResult};
use syntax::expr::atom::{method_call, name};
use syntax::expr::{atom, precedence_1, precedence_2};
use syntax::tree::{Assigned, Assignment, BinaryOperation, Expr, FieldAccess, Name, Span};
use syntax::{comment, expr, tag, tag_and_followed_by};

fn op(input: Span) -> IResult<Span, Span> {
    alt((
        tag_and_followed_by("=", is_not("=")),
        tag("+="),
        tag("-="),
        tag("*="),
        tag("/="),
        tag("%="),
        tag("|="),
        tag("&="),
        tag("^="),
        tag("<<="),
        tag(">>="),
    ))(input)
}

pub fn parse_tail<'a>(left: Expr<'a>, input: Span<'a>) -> IResult<Span<'a>, Expr<'a>> {
    let (input, operator) = match op(input) {
        Ok(ok) => ok,
        _ => return precedence_2::parse_tail(left, input),
    };

    let assigned = match left {
        Expr::FieldAccess(field) => Assigned::Field(field),
        Expr::ArrayAccess(arr) => Assigned::ArrayAccess(arr),
        Expr::Name(name) => Assigned::Name(name),
        _ => return Err(nom::Err::Error((input, ErrorKind::Tag))),
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

pub fn parse(input: Span) -> IResult<Span, Expr> {
    let (input, left) = precedence_2::parse(input)?;
    parse_tail(left, input)
}

#[cfg(test)]
mod tests {
    use syntax::tree::{
        ArrayAccess, Assigned, Assignment, BinaryOperation, ClassType, Expr, FieldAccess, Int,
        LiteralString, Method, MethodCall, Name, ReturnStmt, TypeArg,
    };
    use test_common::{code, span};

    use super::parse;

    #[test]
    fn test_and_assignment() {
        assert_eq!(
            parse(code(
                r#"
a &= b
            "#
                .trim()
            )),
            Ok((
                span(1, 7, ""),
                Expr::Assignment(Assignment {
                    assigned: Box::new(Assigned::Name(Name {
                        name: span(1, 1, "a")
                    })),
                    operator: span(1, 3, "&="),
                    expr: Box::new(Expr::Name(Name {
                        name: span(1, 6, "b")
                    }))
                })
            ))
        );
    }

    #[test]
    fn test_assignment() {
        assert_eq!(
            parse(code(
                r#"
a = b.a += c.d[0][1] *= 1 == 2
            "#
                .trim()
            )),
            Ok((
                span(1, 31, ""),
                Expr::Assignment(Assignment {
                    assigned: Box::new(Assigned::Name(Name {
                        name: span(1, 1, "a")
                    })),
                    operator: span(1, 3, "="),
                    expr: Box::new(Expr::Assignment(Assignment {
                        assigned: Box::new(Assigned::Field(FieldAccess {
                            expr: Box::new(Expr::Name(Name {
                                name: span(1, 5, "b")
                            })),
                            field: Name {
                                name: span(1, 7, "a")
                            }
                        })),
                        operator: span(1, 9, "+="),
                        expr: Box::new(Expr::Assignment(Assignment {
                            assigned: Box::new(Assigned::ArrayAccess(ArrayAccess {
                                expr: Box::new(Expr::ArrayAccess(ArrayAccess {
                                    expr: Box::new(Expr::FieldAccess(FieldAccess {
                                        expr: Box::new(Expr::Name(Name {
                                            name: span(1, 12, "c")
                                        })),
                                        field: Name {
                                            name: span(1, 14, "d")
                                        }
                                    })),
                                    index: Box::new(Expr::Int(Int {
                                        value: span(1, 16, "0")
                                    }))
                                })),
                                index: Box::new(Expr::Int(Int {
                                    value: span(1, 19, "1")
                                }))
                            })),
                            operator: span(1, 22, "*="),
                            expr: Box::new(Expr::BinaryOperation(BinaryOperation {
                                left: Box::new(Expr::Int(Int {
                                    value: span(1, 25, "1")
                                })),
                                operator: span(1, 27, "=="),
                                right: Box::new(Expr::Int(Int {
                                    value: span(1, 30, "2")
                                }))
                            }))
                        }))
                    }))
                })
            ))
        );
    }
}
