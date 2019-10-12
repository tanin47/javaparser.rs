use parse::combinator::{identifier, separated_nonempty_list, symbol};
use parse::id_gen::IdGen;
use parse::statement::variable_declarators;
use parse::tree::{FieldDeclarator, FieldDeclarators, Modifier, Type};
use parse::{expr, tpe, ParseResult, Tokens};
use std::cell::RefCell;

pub fn parse<'def, 'r>(
    input: Tokens<'def, 'r>,
    modifiers: Vec<Modifier<'def>>,
    tpe: Type<'def>,
    id_gen: &mut IdGen,
) -> ParseResult<'def, 'r, FieldDeclarators<'def>> {
    let (input, declarators) =
        separated_nonempty_list(symbol(','), |i| parse_single(i, tpe.clone(), id_gen))(input)?;

    let (input, _) = symbol(';')(input)?;

    Ok((
        input,
        FieldDeclarators {
            modifiers,
            declarators,
        },
    ))
}

fn parse_single<'def: 'r, 'r, 'id_gen_ref>(
    input: Tokens<'def, 'r>,
    tpe: Type<'def>,
    id_gen: &'id_gen_ref mut IdGen,
) -> ParseResult<'def, 'r, FieldDeclarator<'def>> {
    let (input, name) = identifier(input)?;
    let (input, tpe) = tpe::array::parse_tail(input, tpe)?;

    let (input, expr_opt) = match symbol('=')(input) {
        Ok((input, _)) => {
            let (input, expr) = expr::parse(input, id_gen)?;
            (input, Some(expr))
        }
        Err(_) => (input, None),
    };

    Ok((
        input,
        FieldDeclarator {
            tpe: RefCell::new(tpe),
            name,
            expr_opt,
            id: id_gen.get_next("var", name.fragment),
            def_opt: RefCell::new(None),
        },
    ))
}

//#[cfg(test)]
//mod tests {
//    use parse::def::class_body;
//    use parse::tree::{
//        Annotated, ArrayType, ClassBodyItem, ClassType, Expr, FieldDeclarators, Int, Keyword,
//        MarkerAnnotated, Modifier, PrimitiveType, PrimitiveTypeType, Type, VariableDeclarator,
//    };
//    use parse::Tokens;
//    use std::cell::RefCell;
//    use test_common::{generate_tokens, span};
//
//    #[test]
//    fn test_bare() {
//        assert_eq!(
//            class_body::parse_item(&generate_tokens(
//                r#"
//@Anno private int a;
//            "#
//            )),
//            Ok((
//                &[] as Tokens,
//                ClassBodyItem::FieldDeclarators(FieldDeclarators {
//                    modifiers: vec![
//                        Modifier::Annotated(Annotated::Marker(MarkerAnnotated {
//                            class: ClassType {
//                                prefix_opt: None,
//                                name: span(1, 2, "Anno"),
//                                type_args_opt: None,
//                                def_opt: None
//                            }
//                        })),
//                        Modifier::Keyword(Keyword {
//                            name: span(1, 7, "private")
//                        })
//                    ],
//                    declarators: vec![VariableDeclarator {
//                        tpe: RefCell::new(Type::Primitive(PrimitiveType {
//                            name: span(1, 15, "int"),
//                            tpe: PrimitiveTypeType::Int
//                        })),
//                        name: span(1, 19, "a"),
//                        expr_opt: None
//                    }]
//                })
//            ))
//        );
//    }
//
//    #[test]
//    fn test_weird_array() {
//        assert_eq!(
//            class_body::parse_item(&generate_tokens(
//                r#"
//static int[] a, b[];
//            "#
//                .trim()
//            )),
//            Ok((
//                &[] as Tokens,
//                ClassBodyItem::FieldDeclarators(FieldDeclarators {
//                    modifiers: vec![Modifier::Keyword(Keyword {
//                        name: span(1, 1, "static")
//                    })],
//                    declarators: vec![
//                        VariableDeclarator {
//                            tpe: RefCell::new(Type::Array(ArrayType {
//                                tpe: Box::new(Type::Primitive(PrimitiveType {
//                                    name: span(1, 8, "int"),
//                                    tpe: PrimitiveTypeType::Int
//                                })),
//                                size_opt: None
//                            })),
//                            name: span(1, 14, "a"),
//                            expr_opt: None
//                        },
//                        VariableDeclarator {
//                            tpe: RefCell::new(Type::Array(ArrayType {
//                                tpe: Box::new(Type::Array(ArrayType {
//                                    tpe: Box::new(Type::Primitive(PrimitiveType {
//                                        name: span(1, 8, "int"),
//                                        tpe: PrimitiveTypeType::Int
//                                    })),
//                                    size_opt: None
//                                })),
//                                size_opt: None
//                            })),
//                            name: span(1, 17, "b"),
//                            expr_opt: None
//                        },
//                    ]
//                })
//            ))
//        );
//    }
//
//    #[test]
//    fn test_expr() {
//        assert_eq!(
//            class_body::parse_item(&generate_tokens(
//                r#"
//int a = 1;
//            "#
//            )),
//            Ok((
//                &[] as Tokens,
//                ClassBodyItem::FieldDeclarators(FieldDeclarators {
//                    modifiers: vec![],
//                    declarators: vec![VariableDeclarator {
//                        tpe: RefCell::new(Type::Primitive(PrimitiveType {
//                            name: span(1, 1, "int"),
//                            tpe: PrimitiveTypeType::Int
//                        })),
//                        name: span(1, 5, "a"),
//                        expr_opt: Some(Expr::Int(Int {
//                            value: span(1, 9, "1")
//                        }))
//                    }]
//                })
//            ))
//        );
//    }
//
//    #[test]
//    fn test_multiple() {
//        assert_eq!(
//            class_body::parse_item(&generate_tokens(
//                r#"
//int a = 1, b[], c;
//            "#
//            )),
//            Ok((
//                &[] as Tokens,
//                ClassBodyItem::FieldDeclarators(FieldDeclarators {
//                    modifiers: vec![],
//                    declarators: vec![
//                        VariableDeclarator {
//                            tpe: RefCell::new(Type::Primitive(PrimitiveType {
//                                name: span(1, 1, "int"),
//                                tpe: PrimitiveTypeType::Int
//                            })),
//                            name: span(1, 5, "a"),
//                            expr_opt: Some(Expr::Int(Int {
//                                value: span(1, 9, "1")
//                            }))
//                        },
//                        VariableDeclarator {
//                            tpe: RefCell::new(Type::Array(ArrayType {
//                                tpe: Box::new(Type::Primitive(PrimitiveType {
//                                    name: span(1, 1, "int"),
//                                    tpe: PrimitiveTypeType::Int
//                                })),
//                                size_opt: None
//                            })),
//                            name: span(1, 12, "b"),
//                            expr_opt: None
//                        },
//                        VariableDeclarator {
//                            tpe: RefCell::new(Type::Primitive(PrimitiveType {
//                                name: span(1, 1, "int"),
//                                tpe: PrimitiveTypeType::Int
//                            })),
//                            name: span(1, 17, "c"),
//                            expr_opt: None
//                        }
//                    ]
//                })
//            ))
//        );
//    }
//}
