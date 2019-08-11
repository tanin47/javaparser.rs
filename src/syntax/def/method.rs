use nom::character::complete::multispace0;
use nom::character::is_space;
use nom::IResult;

use nom::branch::alt;
use nom::combinator::map;
use nom::multi::{many0, separated_list, separated_nonempty_list};
use syntax::def::{annotateds, modifiers, param, type_params};
use syntax::expr::atom::name;
use syntax::statement::block;
use syntax::tree::{Class, Method, Modifier, Type, TypeParam};
use syntax::tree::{ClassType, Span};
use syntax::{comment, tag, tpe};

fn parse_throws(input: Span) -> IResult<Span, Vec<ClassType>> {
    let (input, _) = match tag("throws")(input) {
        Ok(ok) => ok,
        Err(_) => return Ok((input, vec![])),
    };

    separated_nonempty_list(tag(","), tpe::class::parse_no_array)(input)
}

pub fn parse<'a>(
    input: Span<'a>,
    modifiers: Vec<Modifier<'a>>,
    type_params: Vec<TypeParam<'a>>,
    return_type: Type<'a>,
    name: Span<'a>,
) -> IResult<Span<'a>, Method<'a>> {
    let (input, _) = tag("(")(input)?;
    let (input, params) = separated_list(tag(","), param::parse)(input)?;
    let (input, _) = tag(")")(input)?;

    let (input, throws) = parse_throws(input)?;

    let (input, block_opt) = alt((
        map(block::parse_block, |b| Some(b)),
        map(tag(";"), |_| None),
    ))(input)?;

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
        },
    ))
}

#[cfg(test)]
mod tests {
    use syntax::def::class_body;
    use syntax::tree::{
        Annotated, Block, ClassBodyItem, ClassType, Expr, Int, Keyword, MarkerAnnotated, Method,
        Modifier, Param, PrimitiveType, ReturnStmt, Statement, Type, TypeArg, TypeParam,
    };
    use test_common::{code, primitive, span};

    #[test]
    fn test_abstract() {
        assert_eq!(
            class_body::parse_item(code(
                r#"
@Anno abstract void method() throws Exception, AnotherException;
            "#
                .trim()
            )),
            Ok((
                span(1, 65, ""),
                ClassBodyItem::Method(Method {
                    modifiers: vec![
                        Modifier::Annotated(Annotated::Marker(MarkerAnnotated {
                            name: span(1, 2, "Anno")
                        })),
                        Modifier::Keyword(Keyword {
                            name: span(1, 7, "abstract")
                        })
                    ],
                    return_type: primitive(1, 16, "void"),
                    name: span(1, 21, "method"),
                    type_params: vec![],
                    params: vec![],
                    throws: vec![
                        ClassType {
                            prefix_opt: None,
                            name: span(1, 37, "Exception"),
                            type_args_opt: None
                        },
                        ClassType {
                            prefix_opt: None,
                            name: span(1, 48, "AnotherException"),
                            type_args_opt: None
                        }
                    ],
                    block_opt: None,
                })
            ))
        );
    }

    #[test]
    fn test_method() {
        assert_eq!(
            class_body::parse_item(code(
                r#"
private void method() {
    return 1;
}
            "#
                .trim()
            )),
            Ok((
                span(3, 2, ""),
                ClassBodyItem::Method(Method {
                    modifiers: vec![Modifier::Keyword(Keyword {
                        name: span(1, 1, "private")
                    })],
                    return_type: primitive(1, 9, "void"),
                    name: span(1, 14, "method"),
                    type_params: vec![],
                    params: vec![],
                    throws: vec![],
                    block_opt: Some(Block {
                        stmts: vec![Statement::Return(ReturnStmt {
                            expr_opt: Some(Expr::Int(Int {
                                value: span(2, 12, "1")
                            }))
                        })]
                    }),
                })
            ))
        );
    }

    #[test]
    fn test_method_with_params() {
        assert_eq!(
            class_body::parse_item(code(
                r#"
<A> void method(Test t, A a) {
    return 1;
}
            "#
                .trim()
            )),
            Ok((
                span(3, 2, ""),
                ClassBodyItem::Method(Method {
                    modifiers: vec![],
                    return_type: primitive(1, 5, "void"),
                    name: span(1, 10, "method"),
                    type_params: vec![TypeParam {
                        name: span(1, 2, "A"),
                        extends: vec![],
                    }],
                    params: vec![
                        Param {
                            modifiers: vec![],
                            tpe: Type::Class(ClassType {
                                prefix_opt: None,
                                name: span(1, 17, "Test"),
                                type_args_opt: None
                            }),
                            is_varargs: false,
                            name: span(1, 22, "t"),
                        },
                        Param {
                            modifiers: vec![],
                            tpe: Type::Class(ClassType {
                                prefix_opt: None,
                                name: span(1, 25, "A"),
                                type_args_opt: None
                            }),
                            is_varargs: false,
                            name: span(1, 27, "a"),
                        }
                    ],
                    throws: vec![],
                    block_opt: Some(Block {
                        stmts: vec![Statement::Return(ReturnStmt {
                            expr_opt: Some(Expr::Int(Int {
                                value: span(2, 12, "1")
                            }))
                        })]
                    }),
                })
            ))
        );
    }

    #[test]
    fn test_method_with_type_params() {
        assert_eq!(
            class_body::parse_item(code(
                r#"
<A, B extends A> void method(Test<A> t, B b) {
    return 1;
}
            "#
                .trim()
            )),
            Ok((
                span(3, 2, ""),
                ClassBodyItem::Method(Method {
                    modifiers: vec![],
                    return_type: primitive(1, 18, "void"),
                    name: span(1, 23, "method"),
                    type_params: vec![
                        TypeParam {
                            name: span(1, 2, "A"),
                            extends: vec![]
                        },
                        TypeParam {
                            name: span(1, 5, "B"),
                            extends: vec![ClassType {
                                prefix_opt: None,
                                name: span(1, 15, "A"),
                                type_args_opt: None
                            }]
                        }
                    ],
                    params: vec![
                        Param {
                            modifiers: vec![],
                            tpe: Type::Class(ClassType {
                                prefix_opt: None,
                                name: span(1, 30, "Test"),
                                type_args_opt: Some(vec![TypeArg::Class(ClassType {
                                    prefix_opt: None,
                                    name: span(1, 35, "A"),
                                    type_args_opt: None
                                })])
                            }),
                            is_varargs: false,
                            name: span(1, 38, "t"),
                        },
                        Param {
                            modifiers: vec![],
                            tpe: Type::Class(ClassType {
                                prefix_opt: None,
                                name: span(1, 41, "B"),
                                type_args_opt: None
                            }),
                            is_varargs: false,
                            name: span(1, 43, "b"),
                        }
                    ],
                    throws: vec![],
                    block_opt: Some(Block {
                        stmts: vec![Statement::Return(ReturnStmt {
                            expr_opt: Some(Expr::Int(Int {
                                value: span(2, 12, "1")
                            }))
                        })]
                    }),
                })
            ))
        );
    }
}
