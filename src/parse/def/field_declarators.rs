use parse::combinator::{separated_nonempty_list, symbol};
use parse::statement::variable_declarators;
use parse::tree::{FieldDeclarators, Modifier, Type};
use parse::{ParseResult, Tokens};

pub fn parse<'a>(
    input: Tokens<'a>,
    modifiers: Vec<Modifier<'a>>,
    tpe: Type<'a>,
) -> ParseResult<'a, FieldDeclarators<'a>> {
    let (input, declarators) =
        separated_nonempty_list(symbol(','), variable_declarators::parse_single(tpe))(input)?;

    let (input, _) = symbol(';')(input)?;

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
    use parse::def::class_body;
    use parse::tree::{
        Annotated, ArrayType, ClassBodyItem, ClassType, Expr, FieldDeclarators, Int, Keyword,
        MarkerAnnotated, Modifier, PrimitiveType, PrimitiveTypeType, Type, VariableDeclarator,
    };
    use parse::Tokens;
    use std::cell::RefCell;
    use test_common::{generate_tokens, span};

    #[test]
    fn test_bare() {
        assert_eq!(
            class_body::parse_item(&generate_tokens(
                r#"
@Anno private int a;
            "#
            )),
            Ok((
                &[] as Tokens,
                ClassBodyItem::FieldDeclarators(FieldDeclarators {
                    modifiers: vec![
                        Modifier::Annotated(Annotated::Marker(MarkerAnnotated {
                            class: ClassType {
                                prefix_opt: None,
                                name: span(1, 2, "Anno"),
                                type_args_opt: None,
                                def_opt: None
                            }
                        })),
                        Modifier::Keyword(Keyword {
                            name: span(1, 7, "private")
                        })
                    ],
                    declarators: vec![VariableDeclarator {
                        tpe: RefCell::new(Type::Primitive(PrimitiveType {
                            name: span(1, 15, "int"),
                            tpe: PrimitiveTypeType::Int
                        })),
                        name: span(1, 19, "a"),
                        expr_opt: None
                    }]
                })
            ))
        );
    }

    #[test]
    fn test_weird_array() {
        assert_eq!(
            class_body::parse_item(&generate_tokens(
                r#"
static int[] a, b[];
            "#
                .trim()
            )),
            Ok((
                &[] as Tokens,
                ClassBodyItem::FieldDeclarators(FieldDeclarators {
                    modifiers: vec![Modifier::Keyword(Keyword {
                        name: span(1, 1, "static")
                    })],
                    declarators: vec![
                        VariableDeclarator {
                            tpe: RefCell::new(Type::Array(ArrayType {
                                tpe: Box::new(Type::Primitive(PrimitiveType {
                                    name: span(1, 8, "int"),
                                    tpe: PrimitiveTypeType::Int
                                })),
                                size_opt: None
                            })),
                            name: span(1, 14, "a"),
                            expr_opt: None
                        },
                        VariableDeclarator {
                            tpe: RefCell::new(Type::Array(ArrayType {
                                tpe: Box::new(Type::Array(ArrayType {
                                    tpe: Box::new(Type::Primitive(PrimitiveType {
                                        name: span(1, 8, "int"),
                                        tpe: PrimitiveTypeType::Int
                                    })),
                                    size_opt: None
                                })),
                                size_opt: None
                            })),
                            name: span(1, 17, "b"),
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
            class_body::parse_item(&generate_tokens(
                r#"
int a = 1;
            "#
            )),
            Ok((
                &[] as Tokens,
                ClassBodyItem::FieldDeclarators(FieldDeclarators {
                    modifiers: vec![],
                    declarators: vec![VariableDeclarator {
                        tpe: RefCell::new(Type::Primitive(PrimitiveType {
                            name: span(1, 1, "int"),
                            tpe: PrimitiveTypeType::Int
                        })),
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
            class_body::parse_item(&generate_tokens(
                r#"
int a = 1, b[], c;
            "#
            )),
            Ok((
                &[] as Tokens,
                ClassBodyItem::FieldDeclarators(FieldDeclarators {
                    modifiers: vec![],
                    declarators: vec![
                        VariableDeclarator {
                            tpe: RefCell::new(Type::Primitive(PrimitiveType {
                                name: span(1, 1, "int"),
                                tpe: PrimitiveTypeType::Int
                            })),
                            name: span(1, 5, "a"),
                            expr_opt: Some(Expr::Int(Int {
                                value: span(1, 9, "1")
                            }))
                        },
                        VariableDeclarator {
                            tpe: RefCell::new(Type::Array(ArrayType {
                                tpe: Box::new(Type::Primitive(PrimitiveType {
                                    name: span(1, 1, "int"),
                                    tpe: PrimitiveTypeType::Int
                                })),
                                size_opt: None
                            })),
                            name: span(1, 12, "b"),
                            expr_opt: None
                        },
                        VariableDeclarator {
                            tpe: RefCell::new(Type::Primitive(PrimitiveType {
                                name: span(1, 1, "int"),
                                tpe: PrimitiveTypeType::Int
                            })),
                            name: span(1, 17, "c"),
                            expr_opt: None
                        }
                    ]
                })
            ))
        );
    }
}
