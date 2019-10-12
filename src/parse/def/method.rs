use parse::combinator::{keyword, opt, separated_list, separated_nonempty_list, symbol};
use parse::def::param;
use parse::id_gen::IdGen;
use parse::statement::block;
use parse::tpe::array;
use parse::tree::{ClassType, Method, Modifier, Type, TypeParam};
use parse::{tpe, ParseResult, Tokens};
use std::cell::RefCell;
use tokenize::span::Span;

pub fn parse_throws<'def, 'r>(
    input: Tokens<'def, 'r>,
) -> ParseResult<'def, 'r, Vec<ClassType<'def>>> {
    if let Ok((input, _)) = keyword("throws")(input) {
        separated_nonempty_list(symbol(','), tpe::class::parse_no_array)(input)
    } else {
        Ok((input, vec![]))
    }
}

pub fn parse<'def, 'r>(
    input: Tokens<'def, 'r>,
    modifiers: Vec<Modifier<'def>>,
    type_params: Vec<TypeParam<'def>>,
    return_type: Type<'def>,
    name: Span<'def>,
    id_gen: &mut IdGen,
) -> ParseResult<'def, 'r, Method<'def>> {
    let (input, _) = symbol('(')(input)?;
    let (input, params) = separated_list(symbol(','), |i| param::parse(i, id_gen))(input)?;
    let (input, _) = symbol(')')(input)?;

    let (input, return_type) = array::parse_tail(input, return_type)?;

    let (input, throws) = parse_throws(input)?;

    let (input, block_opt) = if let Ok((input, _)) = symbol(';')(input) {
        (input, None)
    } else {
        let (input, block) = block::parse_block(input, id_gen)?;
        let (input, _) = opt(symbol(';'))(input)?;
        (input, Some(block))
    };

    Ok((
        input,
        Method {
            modifiers,
            type_params,
            return_type,
            name,
            params,
            throws,
            block_opt,
            def_opt: RefCell::new(None),
            id: id_gen.get_next("method", name.fragment),
        },
    ))
}

//#[cfg(test)]
//mod tests {
//    use parse::def::class_body;
//    use parse::tree::{
//        Annotated, ArrayType, Block, ClassBodyItem, ClassType, Expr, Int, Keyword, MarkerAnnotated,
//        Method, Modifier, Param, PrimitiveType, PrimitiveTypeType, ReferenceType, ReturnStmt,
//        Statement, Type, TypeArg, TypeParam, Void,
//    };
//    use parse::Tokens;
//    use test_common::{generate_tokens, primitive, span};
//
//    #[test]
//    fn test_abstract() {
//        assert_eq!(
//            class_body::parse_item(&generate_tokens(
//                r#"
//@Anno abstract void method() throws Exception, AnotherException;
//            "#
//            )),
//            Ok((
//                &[] as Tokens,
//                ClassBodyItem::Method(Method {
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
//                            name: span(1, 7, "abstract")
//                        })
//                    ],
//                    return_type: Type::Void(Void {
//                        span: span(1, 16, "void")
//                    }),
//                    name: span(1, 21, "method"),
//                    type_params: vec![],
//                    params: vec![],
//                    throws: vec![
//                        ClassType {
//                            prefix_opt: None,
//                            name: span(1, 37, "Exception"),
//                            type_args_opt: None,
//                            def_opt: None
//                        },
//                        ClassType {
//                            prefix_opt: None,
//                            name: span(1, 48, "AnotherException"),
//                            type_args_opt: None,
//                            def_opt: None
//                        }
//                    ],
//                    block_opt: None,
//                })
//            ))
//        );
//    }
//
//    #[test]
//    fn test_array_tail() {
//        assert_eq!(
//            class_body::parse_item(&generate_tokens(
//                r#"
//int method()[] {}
//            "#
//            )),
//            Ok((
//                &[] as Tokens,
//                ClassBodyItem::Method(Method {
//                    modifiers: vec![],
//                    return_type: Type::Array(ArrayType {
//                        tpe: Box::new(Type::Primitive(PrimitiveType {
//                            name: span(1, 1, "int"),
//                            tpe: PrimitiveTypeType::Int
//                        })),
//                        size_opt: None
//                    }),
//                    name: span(1, 5, "method"),
//                    type_params: vec![],
//                    params: vec![],
//                    throws: vec![],
//                    block_opt: Some(Block { stmts: vec![] }),
//                })
//            ))
//        );
//    }
//
//    #[test]
//    fn test_method() {
//        assert_eq!(
//            class_body::parse_item(&generate_tokens(
//                r#"
//private void method() {}
//            "#
//            )),
//            Ok((
//                &[] as Tokens,
//                ClassBodyItem::Method(Method {
//                    modifiers: vec![Modifier::Keyword(Keyword {
//                        name: span(1, 1, "private")
//                    })],
//                    return_type: Type::Void(Void {
//                        span: span(1, 9, "void")
//                    }),
//                    name: span(1, 14, "method"),
//                    type_params: vec![],
//                    params: vec![],
//                    throws: vec![],
//                    block_opt: Some(Block { stmts: vec![] }),
//                })
//            ))
//        );
//    }
//
//    #[test]
//    fn test_method_with_params() {
//        assert_eq!(
//            class_body::parse_item(&generate_tokens(
//                r#"
//<A> void method(Test t, A a) {}
//            "#
//            )),
//            Ok((
//                &[] as Tokens,
//                ClassBodyItem::Method(Method {
//                    modifiers: vec![],
//                    return_type: Type::Void(Void {
//                        span: span(1, 5, "void")
//                    }),
//                    name: span(1, 10, "method"),
//                    type_params: vec![TypeParam {
//                        name: span(1, 2, "A"),
//                        extends: vec![],
//                    }],
//                    params: vec![
//                        Param {
//                            modifiers: vec![],
//                            tpe: Type::Class(ClassType {
//                                prefix_opt: None,
//                                name: span(1, 17, "Test"),
//                                type_args_opt: None,
//                                def_opt: None
//                            }),
//                            is_varargs: false,
//                            name: span(1, 22, "t"),
//                        },
//                        Param {
//                            modifiers: vec![],
//                            tpe: Type::Class(ClassType {
//                                prefix_opt: None,
//                                name: span(1, 25, "A"),
//                                type_args_opt: None,
//                                def_opt: None
//                            }),
//                            is_varargs: false,
//                            name: span(1, 27, "a"),
//                        }
//                    ],
//                    throws: vec![],
//                    block_opt: Some(Block { stmts: vec![] }),
//                })
//            ))
//        );
//    }
//
//    #[test]
//    fn test_method_with_type_params() {
//        assert_eq!(
//            class_body::parse_item(&generate_tokens(
//                r#"
//<A, B extends A> void method(Test<A> t, B b) {}
//            "#
//            )),
//            Ok((
//                &[] as Tokens,
//                ClassBodyItem::Method(Method {
//                    modifiers: vec![],
//                    return_type: Type::Void(Void {
//                        span: span(1, 18, "void")
//                    }),
//                    name: span(1, 23, "method"),
//                    type_params: vec![
//                        TypeParam {
//                            name: span(1, 2, "A"),
//                            extends: vec![]
//                        },
//                        TypeParam {
//                            name: span(1, 5, "B"),
//                            extends: vec![ClassType {
//                                prefix_opt: None,
//                                name: span(1, 15, "A"),
//                                type_args_opt: None,
//                                def_opt: None
//                            }]
//                        }
//                    ],
//                    params: vec![
//                        Param {
//                            modifiers: vec![],
//                            tpe: Type::Class(ClassType {
//                                prefix_opt: None,
//                                name: span(1, 30, "Test"),
//                                type_args_opt: Some(vec![TypeArg::Class(ClassType {
//                                    prefix_opt: None,
//                                    name: span(1, 35, "A"),
//                                    type_args_opt: None,
//                                    def_opt: None
//                                })]),
//                                def_opt: None
//                            }),
//                            is_varargs: false,
//                            name: span(1, 38, "t"),
//                        },
//                        Param {
//                            modifiers: vec![],
//                            tpe: Type::Class(ClassType {
//                                prefix_opt: None,
//                                name: span(1, 41, "B"),
//                                type_args_opt: None,
//                                def_opt: None
//                            }),
//                            is_varargs: false,
//                            name: span(1, 43, "b"),
//                        }
//                    ],
//                    throws: vec![],
//                    block_opt: Some(Block { stmts: vec![] }),
//                })
//            ))
//        );
//    }
//}
