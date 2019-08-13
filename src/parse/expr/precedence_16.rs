use either::Either;
use parse::combinator::symbol;
use parse::expr::atom;
use parse::expr::atom::{array_access, method_call, name};
use parse::expr::precedence_15::convert_to_type;
use parse::tree::{ClassExpr, Expr, FieldAccess, Keyword, MethodCall, Super, This, Type};
use parse::{tpe, ParseResult, Tokens};

pub fn parse(input: Tokens) -> ParseResult<Expr> {
    // This doesn't work. Need to rethink it.
    let result = atom::parse(input);

    if let Ok((input, left)) = result {
        parse_tail(left, input)
    } else if let Ok((input, tpe)) = tpe::parse(input) {
        parse_reserved_field_access(tpe, input)
    } else {
        Err(input)
    }
}

fn array_type_tail(input: Tokens) -> ParseResult<()> {
    let (input, _) = symbol('[')(input)?;
    let (input, _) = symbol(']')(input)?;

    Ok((input, ()))
}

pub fn parse_tail<'a>(left: Expr<'a>, input: Tokens<'a>) -> ParseResult<'a, Expr<'a>> {
    let (input, left) = if let Ok(_) = array_type_tail(input) {
        if let Ok(class_type) = convert_to_type(left) {
            let (input, tpe) = tpe::array::parse_tail(input, Type::Class(class_type))?;
            return parse_reserved_field_access(tpe, input);
        } else {
            return Err(input);
        }
    } else {
        array_access::parse_tail(input, left)?
    };

    if let Ok((input, _)) = symbol('.')(input) {
        parse_dot(left, input)
    } else {
        Ok((input, left))
    }
}

fn parse_reserved_field_access<'a>(tpe: Type<'a>, input: Tokens<'a>) -> ParseResult<'a, Expr<'a>> {
    let (input, _) = symbol('.')(input)?;
    let (input, keyword_or_name) = name::parse(input)?;

    let keyword = match keyword_or_name {
        Either::Left(keyword) => keyword,
        Either::Right(_) => return Err(input),
    };
    parse_reserved_field_access_tail(tpe, keyword, input)
}

fn parse_reserved_field_access_tail<'a>(
    tpe: Type<'a>,
    keyword: Keyword<'a>,
    input: Tokens<'a>,
) -> ParseResult<'a, Expr<'a>> {
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

    Ok((input, expr))
}

fn parse_dot<'a>(parent: Expr<'a>, input: Tokens<'a>) -> ParseResult<'a, Expr<'a>> {
    let (input, expr) = if let Ok(_) = symbol('<')(input) {
        let (input, method_call) = method_call::parse(true)(input)?;

        (
            input,
            Expr::MethodCall(MethodCall {
                prefix_opt: Some(Box::new(parent)),
                name: method_call.name,
                type_args_opt: method_call.type_args_opt,
                args: method_call.args,
            }),
        )
    } else {
        let (input, keyword_or_name) = name::parse(input)?;

        match keyword_or_name {
            Either::Left(keyword) => {
                if let Ok(class_type) = convert_to_type(parent) {
                    parse_reserved_field_access_tail(Type::Class(class_type), keyword, input)?
                } else {
                    return Err(input);
                }
            }
            Either::Right(name) => {
                if let Ok(_) = symbol('(')(input) {
                    let (input, method_call) =
                        method_call::parse_tail(input, Some(Box::new(parent)), name.name, None)?;
                    (input, Expr::MethodCall(method_call))
                } else {
                    (
                        input,
                        Expr::FieldAccess(FieldAccess {
                            expr: Box::new(parent),
                            field: name,
                        }),
                    )
                }
            }
        }
    };

    parse_tail(expr, input)
}

#[cfg(test)]
mod tests {
    use test_common::{code, span};

    use super::parse;
    use parse::tree::{
        ArrayType, ClassExpr, ClassType, Expr, FieldAccess, MethodCall, Name, PrimitiveType, Super,
        This, Type,
    };
    use parse::Tokens;

    #[test]
    fn test_super_with_parent() {
        assert_eq!(
            parse(&code(
                r#"
Parent.Test.super.hashCode()
            "#
            )),
            Ok((
                &[] as Tokens,
                Expr::MethodCall(MethodCall {
                    prefix_opt: Some(Box::new(Expr::Super(Super {
                        tpe_opt: Some(Type::Class(ClassType {
                            prefix_opt: Some(Box::new(ClassType {
                                prefix_opt: None,
                                name: span(1, 1, "Parent"),
                                type_args_opt: None
                            })),
                            name: span(1, 8, "Test"),
                            type_args_opt: None
                        })),
                        span: span(1, 13, "super")
                    }))),
                    name: span(1, 19, "hashCode"),
                    type_args_opt: None,
                    args: vec![]
                })
            ))
        );
    }

    #[test]
    fn test_this_with_parent() {
        assert_eq!(
            parse(&code(
                r#"
Parent.Test.this.hashCode()
            "#
            )),
            Ok((
                &[] as Tokens,
                Expr::MethodCall(MethodCall {
                    prefix_opt: Some(Box::new(Expr::This(This {
                        tpe_opt: Some(Type::Class(ClassType {
                            prefix_opt: Some(Box::new(ClassType {
                                prefix_opt: None,
                                name: span(1, 1, "Parent"),
                                type_args_opt: None
                            })),
                            name: span(1, 8, "Test"),
                            type_args_opt: None
                        })),
                        span: span(1, 13, "this")
                    }))),
                    name: span(1, 18, "hashCode"),
                    type_args_opt: None,
                    args: vec![]
                })
            ))
        );
    }

    #[test]
    fn test_class_with_parent() {
        assert_eq!(
            parse(&code(
                r#"
Parent.Test.class.hashCode()
            "#
            )),
            Ok((
                &[] as Tokens,
                Expr::MethodCall(MethodCall {
                    prefix_opt: Some(Box::new(Expr::Class(ClassExpr {
                        tpe: Type::Class(ClassType {
                            prefix_opt: Some(Box::new(ClassType {
                                prefix_opt: None,
                                name: span(1, 1, "Parent"),
                                type_args_opt: None
                            })),
                            name: span(1, 8, "Test"),
                            type_args_opt: None
                        }),
                        span: span(1, 13, "class")
                    }))),
                    name: span(1, 19, "hashCode"),
                    type_args_opt: None,
                    args: vec![]
                })
            ))
        );
    }

    #[test]
    fn test_class() {
        assert_eq!(
            parse(&code(
                r#"
Test.class.hashCode()
            "#
            )),
            Ok((
                &[] as Tokens,
                Expr::MethodCall(MethodCall {
                    prefix_opt: Some(Box::new(Expr::Class(ClassExpr {
                        tpe: Type::Class(ClassType {
                            prefix_opt: None,
                            name: span(1, 1, "Test"),
                            type_args_opt: None
                        }),
                        span: span(1, 6, "class")
                    }))),
                    name: span(1, 12, "hashCode"),
                    type_args_opt: None,
                    args: vec![]
                })
            ))
        );
    }

    #[test]
    fn test_primitive_array_class() {
        assert_eq!(
            parse(&code(
                r#"
byte[].class
            "#
            )),
            Ok((
                &[] as Tokens,
                Expr::Class(ClassExpr {
                    tpe: Type::Array(ArrayType {
                        tpe: Box::new(Type::Primitive(PrimitiveType {
                            name: span(1, 1, "byte")
                        })),
                        size_opt: None
                    }),
                    span: span(1, 8, "class")
                })
            ))
        );
    }

    #[test]
    fn test_array_class_with_parent() {
        assert_eq!(
            parse(&code(
                r#"
Parent.Test[].class
            "#
            )),
            Ok((
                &[] as Tokens,
                Expr::Class(ClassExpr {
                    tpe: Type::Array(ArrayType {
                        tpe: Box::new(Type::Class(ClassType {
                            prefix_opt: Some(Box::new(ClassType {
                                prefix_opt: None,
                                name: span(1, 1, "Parent"),
                                type_args_opt: None
                            })),
                            name: span(1, 8, "Test"),
                            type_args_opt: None
                        })),
                        size_opt: None
                    }),
                    span: span(1, 15, "class")
                })
            ))
        );
    }

    #[test]
    fn test_array_class() {
        assert_eq!(
            parse(&code(
                r#"
Test[].class
            "#
            )),
            Ok((
                &[] as Tokens,
                Expr::Class(ClassExpr {
                    tpe: Type::Array(ArrayType {
                        tpe: Box::new(Type::Class(ClassType {
                            prefix_opt: None,
                            name: span(1, 1, "Test"),
                            type_args_opt: None
                        })),
                        size_opt: None
                    }),
                    span: span(1, 8, "class")
                })
            ))
        );
    }

    #[test]
    fn test_this_field_access() {
        assert_eq!(
            parse(&code(
                r#"
this.field
            "#
            )),
            Ok((
                &[] as Tokens,
                Expr::FieldAccess(FieldAccess {
                    expr: Box::new(Expr::This(This {
                        tpe_opt: None,
                        span: span(1, 1, "this"),
                    })),
                    field: Name {
                        name: span(1, 6, "field")
                    }
                })
            ))
        );
    }

    #[test]
    fn test_super_field_access() {
        assert_eq!(
            parse(&code(
                r#"
super.field
            "#
            )),
            Ok((
                &[] as Tokens,
                Expr::FieldAccess(FieldAccess {
                    expr: Box::new(Expr::Super(Super {
                        tpe_opt: None,
                        span: span(1, 1, "super"),
                    })),
                    field: Name {
                        name: span(1, 7, "field")
                    }
                })
            ))
        );
    }
}
