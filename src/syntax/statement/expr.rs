use nom::bytes::complete::{tag, take, take_till, take_while};
use nom::character::complete::{multispace0, oct_digit0};
use nom::character::is_space;
use nom::IResult;

use nom::branch::alt;
use nom::error::ErrorKind;
use syntax::expr::atom;
use syntax::expr::atom::{name, new_object};
use syntax::expr::precedence_13::unary_pre;
use syntax::tree::{Class, Expr, Method, Statement};
use syntax::tree::{ReturnStmt, Span};
use syntax::{comment, expr};

pub fn parse(input: Span) -> IResult<Span, Statement> {
    let (input, statement) = parse_without_semicolon(input)?;
    let (input, _) = tag(";")(input)?;

    Ok((input, statement))
}

pub fn parse_without_semicolon(input: Span) -> IResult<Span, Statement> {
    let (input, expr) = expr::parse(input)?;
    Ok((input, Statement::Expr(expr)))
}

#[cfg(test)]
mod tests {
    use super::parse;
    use syntax::tree::{
        ArrayAccess, Assigned, Assignment, Expr, FieldAccess, Int, LiteralString, Method,
        MethodCall, Name, ReturnStmt, Statement,
    };
    use test_common::{code, span};

    #[test]
    fn test_return_void() {
        assert_eq!(
            parse(code(
                r#"
a = 123;
            "#
                .trim()
            )),
            Ok((
                span(1, 9, ""),
                Statement::Expr(Expr::Assignment(Assignment {
                    assigned: Box::new(Assigned::Name(Name {
                        name: span(1, 1, "a")
                    })),
                    operator: span(1, 3, "="),
                    expr: Box::new(Expr::Int(Int {
                        value: span(1, 5, "123")
                    }))
                }))
            ))
        );
    }

    #[test]
    fn test_complex() {
        assert_eq!(
            parse(code(
                r#"
a[0].b.c();
            "#
                .trim()
            )),
            Ok((
                span(1, 12, ""),
                Statement::Expr(Expr::MethodCall(MethodCall {
                    prefix_opt: Some(Box::new(Expr::FieldAccess(FieldAccess {
                        expr: Box::new(Expr::ArrayAccess(ArrayAccess {
                            expr: Box::new(Expr::Name(Name {
                                name: span(1, 1, "a")
                            })),
                            index: Box::new(Expr::Int(Int {
                                value: span(1, 3, "0")
                            }))
                        })),
                        field: Name {
                            name: span(1, 6, "b")
                        }
                    }))),
                    name: span(1, 8, "c"),
                    type_args_opt: None,
                    args: vec![]
                }))
            ))
        );
    }
}
