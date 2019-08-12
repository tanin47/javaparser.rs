use either::Either;
use nom::branch::alt;
use nom::bytes::complete::is_not;
use nom::combinator::peek;
use nom::error::ErrorKind;
use nom::sequence::tuple;
use nom::IResult;
use syntax::expr::atom;
use syntax::expr::atom::{array_access, method_call, name};
use syntax::expr::precedence_15::convert_to_type;
use syntax::tree::{Expr, FieldAccess, MethodCall, Name, ReservedFieldAccess, Span, Type};
use syntax::{comment, tag, tpe};

pub fn parse(input: Tokens) -> ParseResult<Expr> {
    // This doesn't work. Need to rethink it.
    let result = atom::parse(input);

    if let Ok((input, left)) = result {
        parse_tail(left, input)
    } else if let Ok((input, tpe)) = tpe::parse(input) {
        parse_reserved_field_access(tpe, input)
    } else {
        Err(nom::Err::Error((input, ErrorKind::Tag)))
    }
}

pub fn parse_tail<'a>(left: Expr<'a>, input: Span<'a>) -> IResult<Span<'a>, Expr<'a>> {
    let (input, left) = if let Ok(_) = tuple((symbol('['), symbol(']')))(input) {
        println!("{:#?}", left);
        if let Ok(class_type) = convert_to_type(left) {
            println!("{:#?}", class_type);
            let (input, tpe) = tpe::array::parse_tail(input, Type::Class(class_type))?;
            return parse_reserved_field_access(tpe, input);
        } else {
            return Err(nom::Err::Error((input, ErrorKind::Tag)));
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

fn parse_reserved_field_access<'a>(tpe: Type<'a>, input: Span<'a>) -> IResult<Span<'a>, Expr<'a>> {
    let (input, _) = symbol('.')(input)?;
    let (input, keyword_or_name) = name::parse(input)?;

    let keyword = match keyword_or_name {
        Either::Left(keyword) => keyword,
        Either::Right(_) => return Err(nom::Err::Error((input, ErrorKind::Tag))),
    };

    Ok((
        input,
        Expr::ReservedFieldAccess(ReservedFieldAccess {
            tpe,
            field: keyword.name,
        }),
    ))
}

fn parse_dot<'a>(parent: Expr<'a>, input: Span<'a>) -> IResult<Span<'a>, Expr<'a>> {
    let (input, expr) = if let Ok((input, _)) = peek(symbol('<'))(input) {
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
                    (
                        input,
                        Expr::ReservedFieldAccess(ReservedFieldAccess {
                            tpe: Type::Class(class_type),
                            field: keyword.name,
                        }),
                    )
                } else {
                    return Err(nom::Err::Error((input, ErrorKind::Tag)));
                }
            }
            Either::Right(name) => {
                if let Ok((input, _)) = peek(symbol('('))(input) {
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
    use syntax::tree::{
        ArrayAccess, ArrayType, Assigned, Assignment, BinaryOperation, Boolean, Cast, ClassType,
        ConstructorReference, Expr, FieldAccess, Int, LiteralString, Method, MethodCall,
        MethodReference, MethodReferencePrimary, Name, PrimitiveType, ReferenceType,
        ReservedFieldAccess, ReturnStmt, Type, TypeArg,
    };
    use test_common::{code, span};

    use super::parse;

    #[test]
    fn test_class_with_parent() {
        assert_eq!(
            parse(code(
                r#"
Parent.Test.class.hashCode()
            "#
                .trim()
            )),
            Ok((
                span(1, 29, ""),
                Expr::MethodCall(MethodCall {
                    prefix_opt: Some(Box::new(Expr::ReservedFieldAccess(ReservedFieldAccess {
                        tpe: Type::Class(ClassType {
                            prefix_opt: Some(Box::new(ClassType {
                                prefix_opt: None,
                                name: span(1, 1, "Parent"),
                                type_args_opt: None
                            })),
                            name: span(1, 8, "Test"),
                            type_args_opt: None
                        }),
                        field: span(1, 13, "class")
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
            parse(code(
                r#"
Test.class.hashCode()
            "#
                .trim()
            )),
            Ok((
                span(1, 22, ""),
                Expr::MethodCall(MethodCall {
                    prefix_opt: Some(Box::new(Expr::ReservedFieldAccess(ReservedFieldAccess {
                        tpe: Type::Class(ClassType {
                            prefix_opt: None,
                            name: span(1, 1, "Test"),
                            type_args_opt: None
                        }),
                        field: span(1, 6, "class")
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
            parse(code(
                r#"
byte[].class
            "#
                .trim()
            )),
            Ok((
                span(1, 13, ""),
                Expr::ReservedFieldAccess(ReservedFieldAccess {
                    tpe: Type::Array(ArrayType {
                        tpe: Box::new(Type::Primitive(PrimitiveType {
                            name: span(1, 1, "byte")
                        })),
                        size_opt: None
                    }),
                    field: span(1, 8, "class")
                })
            ))
        );
    }

    #[test]
    fn test_array_class_with_parent() {
        assert_eq!(
            parse(code(
                r#"
Parent.Test[].class
            "#
                .trim()
            )),
            Ok((
                span(1, 20, ""),
                Expr::ReservedFieldAccess(ReservedFieldAccess {
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
                    field: span(1, 15, "class")
                })
            ))
        );
    }

    #[test]
    fn test_array_class() {
        assert_eq!(
            parse(code(
                r#"
Test[].class
            "#
                .trim()
            )),
            Ok((
                span(1, 13, ""),
                Expr::ReservedFieldAccess(ReservedFieldAccess {
                    tpe: Type::Array(ArrayType {
                        tpe: Box::new(Type::Class(ClassType {
                            prefix_opt: None,
                            name: span(1, 1, "Test"),
                            type_args_opt: None
                        })),
                        size_opt: None
                    }),
                    field: span(1, 8, "class")
                })
            ))
        );
    }
}
