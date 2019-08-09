use nom::bytes::complete::{tag, take, take_till, take_while};
use nom::character::complete::{alpha1, alphanumeric0, multispace0};
use nom::character::{is_alphanumeric, is_space};
use nom::{Compare, IResult, InputLength, InputTake, InputTakeAtPosition};

use nom::branch::alt;
use nom::multi::{many1, separated_list, separated_nonempty_list};
use nom::sequence::{preceded, tuple};
use std::slice;
use syntax::def::{annotateds, modifiers};
use syntax::statement::variable_declarators;
use syntax::tree::{
    Class, FieldDeclarators, Method, Statement, Type, VariableDeclarator, VariableDeclarators,
};
use syntax::tree::{ReturnStmt, Span};
use syntax::{comment, expr, tpe};

pub fn parse(input: Span) -> IResult<Span, FieldDeclarators> {
    let (input, modifiers) = modifiers::parse(input)?;
    let (input, tpe) = tpe::parse(input)?;

    let (input, declarators) =
        separated_nonempty_list(tag(","), variable_declarators::parse_single(tpe))(input)?;

    let (input, _) = tag(";")(input)?;

    Ok((
        input,
        FieldDeclarators {
            modifiers,
            declarators,
        },
    ))
}

#[cfg(test)]
mod tests {
    use super::parse;
    use syntax::tree::{
        Annotated, ArrayType, Expr, FieldDeclarators, Int, Keyword, MarkerAnnotated, Modifier,
        PrimitiveType, Type, VariableDeclarator,
    };
    use test_common::{code, span};

    #[test]
    fn test_bare() {
        assert_eq!(
            parse(code(
                r#"
@Anno private int a;
            "#
                .trim()
            )),
            Ok((
                span(1, 21, ""),
                FieldDeclarators {
                    modifiers: vec![
                        Modifier::Annotated(Annotated::Marker(MarkerAnnotated {
                            name: span(1, 2, "Anno")
                        })),
                        Modifier::Keyword(Keyword {
                            name: span(1, 7, "private")
                        })
                    ],
                    declarators: vec![VariableDeclarator {
                        tpe: Type::Primitive(PrimitiveType {
                            name: span(1, 15, "int"),
                        }),
                        name: span(1, 19, "a"),
                        expr_opt: None
                    }]
                }
            ))
        );
    }

    #[test]
    fn test_weird_array() {
        assert_eq!(
            parse(code(
                r#"
static int[] a, b[];
            "#
                .trim()
            )),
            Ok((
                span(1, 21, ""),
                FieldDeclarators {
                    modifiers: vec![Modifier::Keyword(Keyword {
                        name: span(1, 1, "static")
                    })],
                    declarators: vec![
                        VariableDeclarator {
                            tpe: Type::Array(ArrayType {
                                tpe: Box::new(Type::Primitive(PrimitiveType {
                                    name: span(1, 8, "int"),
                                })),
                                size_opt: None
                            }),
                            name: span(1, 14, "a"),
                            expr_opt: None
                        },
                        VariableDeclarator {
                            tpe: Type::Array(ArrayType {
                                tpe: Box::new(Type::Array(ArrayType {
                                    tpe: Box::new(Type::Primitive(PrimitiveType {
                                        name: span(1, 8, "int"),
                                    })),
                                    size_opt: None
                                })),
                                size_opt: None
                            }),
                            name: span(1, 17, "b"),
                            expr_opt: None
                        },
                    ]
                }
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
                FieldDeclarators {
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
                }
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
                FieldDeclarators {
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
                }
            ))
        );
    }
}
