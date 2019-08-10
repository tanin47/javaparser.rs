use nom::{Compare, IResult, InputLength, InputTake, InputTakeAtPosition};

use nom::error::ErrorKind;
use nom::multi::separated_nonempty_list;
use std::slice;
use syntax::def::{annotateds, modifiers};
use syntax::expr::atom::name;
use syntax::tree::{
    Class, Method, StandaloneVariableDeclarator, Statement, Type, VariableDeclarator,
    VariableDeclarators,
};
use syntax::tree::{ReturnStmt, Span};
use syntax::{comment, expr, tag, tpe};

pub fn parse_single<'a>(
    original_tpe: Type<'a>,
) -> impl Fn(Span<'a>) -> IResult<Span<'a>, VariableDeclarator<'a>> {
    move |input: Span<'a>| {
        let (input, name) = name::identifier(input)?;

        let (input, tpe) = tpe::array::parse_tail(input, original_tpe.clone())?;

        let (input, expr_opt) = match tag("=")(input) {
            Ok((input, _)) => {
                let (input, expr) = expr::parse(input)?;
                (input, Some(expr))
            }
            Err(_) => (input, None),
        };

        Ok((
            input,
            VariableDeclarator {
                tpe,
                name,
                expr_opt,
            },
        ))
    }
}

pub fn parse_standalone(input: Span) -> IResult<Span, StandaloneVariableDeclarator> {
    let (input, modifiers) = modifiers::parse(input)?;
    let (input, tpe) = tpe::parse(input)?;
    let (input, declarator) = parse_single(tpe)(input)?;

    Ok((
        input,
        StandaloneVariableDeclarator {
            modifiers,
            tpe: declarator.tpe,
            name: declarator.name,
            expr_opt: declarator.expr_opt,
        },
    ))
}

pub fn parse_without_semicolon(input: Span) -> IResult<Span, Statement> {
    let (input, modifiers) = modifiers::parse(input)?;
    let (input, tpe) = tpe::parse(input)?;

    let (input, declarators) = separated_nonempty_list(tag(","), parse_single(tpe))(input)?;

    Ok((
        input,
        Statement::VariableDeclarators(VariableDeclarators {
            modifiers,
            declarators,
        }),
    ))
}

pub fn parse(input: Span) -> IResult<Span, Statement> {
    let (input, declarators) = parse_without_semicolon(input)?;
    let (input, _) = tag(";")(input)?;

    Ok((input, declarators))
}

#[cfg(test)]
mod tests {
    use super::parse;
    use syntax::tree::{
        Annotated, ArrayType, Expr, Int, LiteralString, MarkerAnnotated, Method, Modifier,
        PrimitiveType, ReturnStmt, Statement, Type, VariableDeclarator, VariableDeclarators,
    };
    use test_common::{code, span};

    #[test]
    fn test_bare() {
        assert_eq!(
            parse(code(
                r#"
@Anno int a;
            "#
                .trim()
            )),
            Ok((
                span(1, 13, ""),
                Statement::VariableDeclarators(VariableDeclarators {
                    modifiers: vec![Modifier::Annotated(Annotated::Marker(MarkerAnnotated {
                        name: span(1, 2, "Anno")
                    }))],
                    declarators: vec![VariableDeclarator {
                        tpe: Type::Primitive(PrimitiveType {
                            name: span(1, 7, "int"),
                        }),
                        name: span(1, 11, "a"),
                        expr_opt: None
                    }]
                })
            ))
        );
    }

    #[test]
    fn test_weird_array() {
        assert_eq!(
            parse(code(
                r#"
int[] a, b[];
            "#
                .trim()
            )),
            Ok((
                span(1, 14, ""),
                Statement::VariableDeclarators(VariableDeclarators {
                    modifiers: vec![],
                    declarators: vec![
                        VariableDeclarator {
                            tpe: Type::Array(ArrayType {
                                tpe: Box::new(Type::Primitive(PrimitiveType {
                                    name: span(1, 1, "int"),
                                })),
                                size_opt: None
                            }),
                            name: span(1, 7, "a"),
                            expr_opt: None
                        },
                        VariableDeclarator {
                            tpe: Type::Array(ArrayType {
                                tpe: Box::new(Type::Array(ArrayType {
                                    tpe: Box::new(Type::Primitive(PrimitiveType {
                                        name: span(1, 1, "int"),
                                    })),
                                    size_opt: None
                                })),
                                size_opt: None
                            }),
                            name: span(1, 10, "b"),
                            expr_opt: None
                        },
                    ]
                })
            ))
        );
    }

    #[test]
    fn test_expr() {
        assert_eq!(
            parse(code(
                r#"
int a = 1;
            "#
                .trim()
            )),
            Ok((
                span(1, 11, ""),
                Statement::VariableDeclarators(VariableDeclarators {
                    modifiers: vec![],
                    declarators: vec![VariableDeclarator {
                        tpe: Type::Primitive(PrimitiveType {
                            name: span(1, 1, "int"),
                        }),
                        name: span(1, 5, "a"),
                        expr_opt: Some(Expr::Int(Int {
                            value: span(1, 9, "1")
                        }))
                    }]
                })
            ))
        );
    }

    #[test]
    fn test_multiple() {
        assert_eq!(
            parse(code(
                r#"
int a = 1, b[], c;
            "#
                .trim()
            )),
            Ok((
                span(1, 19, ""),
                Statement::VariableDeclarators(VariableDeclarators {
                    modifiers: vec![],
                    declarators: vec![
                        VariableDeclarator {
                            tpe: Type::Primitive(PrimitiveType {
                                name: span(1, 1, "int"),
                            }),
                            name: span(1, 5, "a"),
                            expr_opt: Some(Expr::Int(Int {
                                value: span(1, 9, "1")
                            }))
                        },
                        VariableDeclarator {
                            tpe: Type::Array(ArrayType {
                                tpe: Box::new(Type::Primitive(PrimitiveType {
                                    name: span(1, 1, "int"),
                                })),
                                size_opt: None
                            }),
                            name: span(1, 12, "b"),
                            expr_opt: None
                        },
                        VariableDeclarator {
                            tpe: Type::Primitive(PrimitiveType {
                                name: span(1, 1, "int"),
                            }),
                            name: span(1, 17, "c"),
                            expr_opt: None
                        }
                    ]
                })
            ))
        );
    }
}
