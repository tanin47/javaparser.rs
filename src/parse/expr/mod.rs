use parse::combinator::symbol2;
use parse::tree::{Expr, MethodReferencePrimary, Type};
use parse::{tpe, ParseResult, Tokens};

pub mod atom;
pub mod precedence_1;
pub mod precedence_10;
pub mod precedence_11;
pub mod precedence_12;
pub mod precedence_13;
pub mod precedence_14;
pub mod precedence_15;
pub mod precedence_16;
pub mod precedence_2;
pub mod precedence_3;
pub mod precedence_4;
pub mod precedence_5;
pub mod precedence_6;
pub mod precedence_7;
pub mod precedence_8;
pub mod precedence_9;

pub fn parse<'def, 'r>(input: Tokens<'def, 'r>) -> ParseResult<'def, 'r, Expr<'def>> {
    if let Ok((input, tpe)) = tpe::parse(input) {
        if let Ok(_) = symbol2(':', ':')(input) {
            match tpe {
                Type::Array(arr) => {
                    return precedence_15::parse_tail(MethodReferencePrimary::Array(arr), input)
                }
                Type::Class(class) => {
                    if tpe::class::contains_type_args(&class) {
                        return precedence_15::parse_tail(
                            MethodReferencePrimary::Class(class),
                            input,
                        );
                    }
                }
                _ => (),
            }
        }
    }
    precedence_1::parse(input)
}

#[cfg(test)]
mod tests {
    use test_common::{generate_tokens, span};

    use super::parse;
    use parse::tree::{
        ArrayType, BinaryOperation, Boolean, ClassExpr, ClassType, ConstructorReference, Expr,
        FieldAccess, MethodCall, MethodReference, MethodReferencePrimary, Name, PrimitiveType,
        PrimitiveTypeType, ReferenceType, Type, TypeArg,
    };
    use parse::Tokens;

    #[test]
    fn test_method_ref_int_array() {
        assert_eq!(
            parse(&generate_tokens(
                r#"
int[]::size
            "#
            )),
            Ok((
                &[] as Tokens,
                Expr::MethodReference(MethodReference {
                    primary: MethodReferencePrimary::Array(ArrayType {
                        tpe: Box::new(Type::Primitive(PrimitiveType {
                            name: span(1, 1, "int"),
                            tpe: PrimitiveTypeType::Int
                        })),
                        size_opt: None
                    }),
                    type_args_opt: None,
                    name: span(1, 8, "size")
                })
            ))
        );
    }

    #[test]
    fn test_constructor_ref() {
        assert_eq!(
            parse(&generate_tokens(
                r#"
Test<A>::new
            "#
            )),
            Ok((
                &[] as Tokens,
                Expr::ConstructorReference(ConstructorReference {
                    tpe: ReferenceType::Class(ClassType {
                        prefix_opt: None,
                        name: span(1, 1, "Test"),
                        type_args_opt: Some(vec![TypeArg::Class(ClassType {
                            prefix_opt: None,
                            name: span(1, 6, "A"),
                            type_args_opt: None,
                            def_opt: None
                        })]),
                        def_opt: None
                    }),
                    type_args_opt: None,
                })
            ))
        );
    }

    #[test]
    fn test_class() {
        assert_eq!(
            parse(&generate_tokens(
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
                            type_args_opt: None,
                            def_opt: None
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
    fn test_parenthesized() {
        assert_eq!(
            parse(&generate_tokens(
                r#"
(true || false) && t.a || false
            "#
            )),
            Ok((
                &[] as Tokens,
                Expr::BinaryOperation(BinaryOperation {
                    left: Box::new(Expr::BinaryOperation(BinaryOperation {
                        left: Box::new(Expr::BinaryOperation(BinaryOperation {
                            left: Box::new(Expr::Boolean(Boolean {
                                value: span(1, 2, "true")
                            })),
                            operator: span(1, 7, "||"),
                            right: Box::new(Expr::Boolean(Boolean {
                                value: span(1, 10, "false")
                            })),
                        })),
                        operator: span(1, 17, "&&"),
                        right: Box::new(Expr::FieldAccess(FieldAccess {
                            expr: Box::new(Expr::Name(Name {
                                name: span(1, 20, "t")
                            })),
                            field: Name {
                                name: span(1, 22, "a")
                            }
                        }))
                    })),
                    operator: span(1, 24, "||"),
                    right: Box::new(Expr::Boolean(Boolean {
                        value: span(1, 27, "false")
                    }))
                })
            ))
        );
    }
}
