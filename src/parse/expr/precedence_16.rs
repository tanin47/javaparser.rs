use either::Either;
use parse::combinator::symbol;
use parse::expr::atom;
use parse::expr::atom::{array_access, invocation, name, new_object};
use parse::expr::precedence_15::convert_to_type;
use parse::id_gen::IdGen;
use parse::tree::{
    ClassExpr, Expr, FieldAccess, FieldAccessPrefix, Keyword, MethodCall, Super,
    SuperConstructorCall, This, Type,
};
use parse::{tpe, ParseResult, Tokens};
use std::cell::RefCell;

pub fn parse<'def, 'r>(
    input: Tokens<'def, 'r>,
    id_gen: &mut IdGen,
) -> ParseResult<'def, 'r, Expr<'def>> {
    // This doesn't work. Need to rethink it.
    let result = atom::parse(input, id_gen);

    if let Ok((input, left)) = result {
        parse_tail(left, input, id_gen)
    } else if let Ok((input, tpe)) = tpe::parse(input) {
        parse_reserved_field_access(tpe, input, id_gen)
    } else {
        Err(input)
    }
}

fn array_type_tail<'def, 'r>(input: Tokens<'def, 'r>) -> ParseResult<'def, 'r, ()> {
    let (input, _) = symbol('[')(input)?;
    let (input, _) = symbol(']')(input)?;

    Ok((input, ()))
}

pub fn parse_tail<'def, 'r>(
    left: Expr<'def>,
    input: Tokens<'def, 'r>,
    id_gen: &mut IdGen,
) -> ParseResult<'def, 'r, Expr<'def>> {
    let (input, left) = if let Ok(_) = array_type_tail(input) {
        if let Ok(class_type) = convert_to_type(left) {
            let (input, tpe) = tpe::array::parse_tail(input, Type::Class(class_type))?;
            return parse_reserved_field_access(tpe, input, id_gen);
        } else {
            return Err(input);
        }
    } else {
        array_access::parse_tail(input, left, id_gen)?
    };

    if let Ok((input, _)) = symbol('.')(input) {
        parse_dot(left, input, id_gen)
    } else {
        Ok((input, left))
    }
}

fn parse_reserved_field_access<'def, 'r>(
    tpe: Type<'def>,
    input: Tokens<'def, 'r>,
    id_gen: &mut IdGen,
) -> ParseResult<'def, 'r, Expr<'def>> {
    let (input, _) = symbol('.')(input)?;
    let (input, keyword_or_name) = name::parse(input)?;

    let keyword = match keyword_or_name {
        Either::Left(keyword) => keyword,
        Either::Right(_) => return Err(input),
    };
    parse_reserved_field_access_tail(tpe, keyword, input, id_gen)
}

fn parse_reserved_field_access_tail<'def, 'r>(
    tpe: Type<'def>,
    keyword: Keyword<'def>,
    input: Tokens<'def, 'r>,
    id_gen: &mut IdGen,
) -> ParseResult<'def, 'r, Expr<'def>> {
    let expr = match keyword.name.fragment {
        "this" => Expr::This(This {
            tpe_opt: Some(tpe),
            span: keyword.name,
        }),
        "super" => Expr::Super(Super {
            tpe_opt: Some(tpe),
            span: keyword.name,
        }),
        "class" => Expr::Class(ClassExpr {
            tpe,
            span: keyword.name,
        }),
        _ => return Err(input),
    };

    parse_tail(expr, input, id_gen)
}

fn parse_dot<'def, 'r>(
    parent: Expr<'def>,
    input: Tokens<'def, 'r>,
    id_gen: &mut IdGen,
) -> ParseResult<'def, 'r, Expr<'def>> {
    let (input, expr) = if let Ok(_) = symbol('<')(input) {
        invocation::parse(input, Some(parent), id_gen)?
    } else {
        let (input, keyword_or_name) = name::parse(input)?;

        if let Ok(_) = symbol('(')(input) {
            invocation::parse_tail(input, Some(parent), keyword_or_name, None, id_gen)?
        } else {
            match keyword_or_name {
                Either::Left(keyword) => {
                    if keyword.name.fragment == "new" {
                        new_object::parse_tail(Some(parent), input, id_gen)?
                    } else if let Ok(class_type) = convert_to_type(parent) {
                        parse_reserved_field_access_tail(
                            Type::Class(class_type),
                            keyword,
                            input,
                            id_gen,
                        )?
                    } else {
                        return Err(input);
                    }
                }
                Either::Right(name) => (
                    input,
                    Expr::FieldAccess(FieldAccess {
                        prefix: RefCell::new(Box::new(FieldAccessPrefix::Expr(parent))),
                        name: name.name,
                        def_opt: RefCell::new(None),
                    }),
                ),
            }
        }
    };

    parse_tail(expr, input, id_gen)
}

//#[cfg(test)]
//mod tests {
//    use test_common::{generate_tokens, span};
//
//    use super::parse;
//    use parse::tree::{
//        ArrayType, ClassExpr, ClassType, EnclosingType, Expr, FieldAccess, MethodCall, Name,
//        NewObject, PrimitiveType, PrimitiveTypeType, Super, SuperConstructorCall, This, Type,
//    };
//    use parse::Tokens;
//    use std::cell::RefCell;
//
//    #[test]
//    fn test_dot_new_member() {
//        assert_eq!(
//            parse(&generate_tokens(
//                r#"
//a.new Test()
//            "#
//            )),
//            Ok((
//                &[] as Tokens,
//                Expr::NewObject(NewObject {
//                    prefix_opt: Some(Box::new(Expr::Name(Name {
//                        name: span(1, 1, "a")
//                    }))),
//                    tpe: ClassType {
//                        prefix_opt: None,
//                        name: span(1, 7, "Test"),
//                        type_args_opt: None,
//                        def_opt: None
//                    },
//                    constructor_type_args_opt: None,
//                    args: vec![],
//                    body_opt: None
//                })
//            ))
//        );
//    }
//
//    #[test]
//    fn test_name_super_constructor_call() {
//        assert_eq!(
//            parse(&generate_tokens(
//                r#"
//test.super()
//            "#
//            )),
//            Ok((
//                &[] as Tokens,
//                Expr::SuperConstructorCall(SuperConstructorCall {
//                    prefix_opt: Some(Box::new(Expr::Name(Name {
//                        name: span(1, 1, "test")
//                    }))),
//                    type_args_opt: None,
//                    name: span(1, 6, "super"),
//                    args: vec![]
//                })
//            ))
//        );
//    }
//
//    #[test]
//    fn test_super_constructor_call() {
//        assert_eq!(
//            parse(&generate_tokens(
//                r#"
//Parent.Test.this.super()
//            "#
//            )),
//            Ok((
//                &[] as Tokens,
//                Expr::SuperConstructorCall(SuperConstructorCall {
//                    prefix_opt: Some(Box::new(Expr::This(This {
//                        tpe_opt: Some(Type::Class(ClassType {
//                            prefix_opt: Some(Box::new(EnclosingType::Class(ClassType {
//                                prefix_opt: None,
//                                name: span(1, 1, "Parent"),
//                                type_args_opt: None,
//                                def_opt: None
//                            }))),
//                            name: span(1, 8, "Test"),
//                            type_args_opt: None,
//                            def_opt: None
//                        })),
//                        span: span(1, 13, "this")
//                    }))),
//                    type_args_opt: None,
//                    name: span(1, 18, "super"),
//                    args: vec![]
//                })
//            ))
//        );
//    }
//
//    #[test]
//    fn test_super_with_parent() {
//        assert_eq!(
//            parse(&generate_tokens(
//                r#"
//Parent.Test.super.hashCode()
//            "#
//            )),
//            Ok((
//                &[] as Tokens,
//                Expr::MethodCall(MethodCall {
//                    prefix_opt: Some(Box::new(Expr::Super(Super {
//                        tpe_opt: Some(Type::Class(ClassType {
//                            prefix_opt: Some(Box::new(EnclosingType::Class(ClassType {
//                                prefix_opt: None,
//                                name: span(1, 1, "Parent"),
//                                type_args_opt: None,
//                                def_opt: None
//                            }))),
//                            name: span(1, 8, "Test"),
//                            type_args_opt: None,
//                            def_opt: None
//                        })),
//                        span: span(1, 13, "super")
//                    }))),
//                    name: span(1, 19, "hashCode"),
//                    type_args_opt: None,
//                    args: vec![]
//                })
//            ))
//        );
//    }
//
//    #[test]
//    fn test_this_with_parent() {
//        assert_eq!(
//            parse(&generate_tokens(
//                r#"
//Parent.Test.this.hashCode()
//            "#
//            )),
//            Ok((
//                &[] as Tokens,
//                Expr::MethodCall(MethodCall {
//                    prefix_opt: Some(Box::new(Expr::This(This {
//                        tpe_opt: Some(Type::Class(ClassType {
//                            prefix_opt: Some(Box::new(EnclosingType::Class(ClassType {
//                                prefix_opt: None,
//                                name: span(1, 1, "Parent"),
//                                type_args_opt: None,
//                                def_opt: None
//                            }))),
//                            name: span(1, 8, "Test"),
//                            type_args_opt: None,
//                            def_opt: None
//                        })),
//                        span: span(1, 13, "this")
//                    }))),
//                    name: span(1, 18, "hashCode"),
//                    type_args_opt: None,
//                    args: vec![]
//                })
//            ))
//        );
//    }
//
//    #[test]
//    fn test_class_with_parent() {
//        assert_eq!(
//            parse(&generate_tokens(
//                r#"
//Parent.Test.class.hashCode()
//            "#
//            )),
//            Ok((
//                &[] as Tokens,
//                Expr::MethodCall(MethodCall {
//                    prefix_opt: Some(Box::new(Expr::Class(ClassExpr {
//                        tpe: Type::Class(ClassType {
//                            prefix_opt: Some(Box::new(EnclosingType::Class(ClassType {
//                                prefix_opt: None,
//                                name: span(1, 1, "Parent"),
//                                type_args_opt: None,
//                                def_opt: None
//                            }))),
//                            name: span(1, 8, "Test"),
//                            type_args_opt: None,
//                            def_opt: None
//                        }),
//                        span: span(1, 13, "class")
//                    }))),
//                    name: span(1, 19, "hashCode"),
//                    type_args_opt: None,
//                    args: vec![]
//                })
//            ))
//        );
//    }
//
//    #[test]
//    fn test_class() {
//        assert_eq!(
//            parse(&generate_tokens(
//                r#"
//Test.class.hashCode()
//            "#
//            )),
//            Ok((
//                &[] as Tokens,
//                Expr::MethodCall(MethodCall {
//                    prefix_opt: Some(Box::new(Expr::Class(ClassExpr {
//                        tpe: Type::Class(ClassType {
//                            prefix_opt: None,
//                            name: span(1, 1, "Test"),
//                            type_args_opt: None,
//                            def_opt: None
//                        }),
//                        span: span(1, 6, "class")
//                    }))),
//                    name: span(1, 12, "hashCode"),
//                    type_args_opt: None,
//                    args: vec![]
//                })
//            ))
//        );
//    }
//
//    #[test]
//    fn test_primitive_class() {
//        assert_eq!(
//            parse(&generate_tokens(
//                r#"
//char.class.hashCode()
//            "#
//            )),
//            Ok((
//                &[] as Tokens,
//                Expr::MethodCall(MethodCall {
//                    prefix_opt: Some(Box::new(Expr::Class(ClassExpr {
//                        tpe: Type::Primitive(PrimitiveType {
//                            name: span(1, 1, "char"),
//                            tpe: PrimitiveTypeType::Char
//                        }),
//                        span: span(1, 6, "class")
//                    }))),
//                    type_args_opt: None,
//                    name: span(1, 12, "hashCode"),
//                    args: vec![]
//                })
//            ))
//        );
//    }
//
//    #[test]
//    fn test_primitive_array_class() {
//        assert_eq!(
//            parse(&generate_tokens(
//                r#"
//byte[].class.hashCode()
//            "#
//            )),
//            Ok((
//                &[] as Tokens,
//                Expr::MethodCall(MethodCall {
//                    prefix_opt: Some(Box::new(Expr::Class(ClassExpr {
//                        tpe: Type::Array(ArrayType {
//                            tpe: Box::new(Type::Primitive(PrimitiveType {
//                                name: span(1, 1, "byte"),
//                                tpe: PrimitiveTypeType::Byte
//                            })),
//                            size_opt: None
//                        }),
//                        span: span(1, 8, "class")
//                    }))),
//                    type_args_opt: None,
//                    name: span(1, 14, "hashCode"),
//                    args: vec![]
//                })
//            ))
//        );
//    }
//
//    #[test]
//    fn test_array_class_with_parent() {
//        assert_eq!(
//            parse(&generate_tokens(
//                r#"
//Parent.Test[].class.hashCode()
//            "#
//            )),
//            Ok((
//                &[] as Tokens,
//                Expr::MethodCall(MethodCall {
//                    prefix_opt: Some(Box::new(Expr::Class(ClassExpr {
//                        tpe: Type::Array(ArrayType {
//                            tpe: Box::new(Type::Class(ClassType {
//                                prefix_opt: Some(Box::new(EnclosingType::Class(ClassType {
//                                    prefix_opt: None,
//                                    name: span(1, 1, "Parent"),
//                                    type_args_opt: None,
//                                    def_opt: None
//                                }))),
//                                name: span(1, 8, "Test"),
//                                type_args_opt: None,
//                                def_opt: None
//                            })),
//                            size_opt: None
//                        }),
//                        span: span(1, 15, "class")
//                    }))),
//                    type_args_opt: None,
//                    name: span(1, 21, "hashCode"),
//                    args: vec![]
//                })
//            ))
//        );
//    }
//
//    #[test]
//    fn test_array_class() {
//        assert_eq!(
//            parse(&generate_tokens(
//                r#"
//Test[].class
//            "#
//            )),
//            Ok((
//                &[] as Tokens,
//                Expr::Class(ClassExpr {
//                    tpe: Type::Array(ArrayType {
//                        tpe: Box::new(Type::Class(ClassType {
//                            prefix_opt: None,
//                            name: span(1, 1, "Test"),
//                            type_args_opt: None,
//                            def_opt: None
//                        })),
//                        size_opt: None
//                    }),
//                    span: span(1, 8, "class")
//                })
//            ))
//        );
//    }
//
//    #[test]
//    fn test_this_field_access() {
//        assert_eq!(
//            parse(&generate_tokens(
//                r#"
//this.field
//            "#
//            )),
//            Ok((
//                &[] as Tokens,
//                Expr::FieldAccess(FieldAccess {
//                    expr: Box::new(Expr::This(This {
//                        tpe_opt: None,
//                        span: span(1, 1, "this"),
//                    })),
//                    field: Name {
//                        name: span(1, 6, "field")
//                    },
//                    tpe_opt: RefCell::new(None)
//                })
//            ))
//        );
//    }
//
//    #[test]
//    fn test_super_field_access() {
//        assert_eq!(
//            parse(&generate_tokens(
//                r#"
//super.field
//            "#
//            )),
//            Ok((
//                &[] as Tokens,
//                Expr::FieldAccess(FieldAccess {
//                    expr: Box::new(Expr::Super(Super {
//                        tpe_opt: None,
//                        span: span(1, 1, "super"),
//                    })),
//                    field: Name {
//                        name: span(1, 7, "field")
//                    },
//                    tpe_opt: RefCell::new(None)
//                })
//            ))
//        );
//    }
//}
