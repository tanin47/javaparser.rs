use parse::combinator::{keyword, opt, symbol};
use parse::expr::atom::array_initializer;
use parse::id_gen::IdGen;
use parse::tree::{ArrayType, ClassType, Expr, NewArray, Type, NATIVE_ARRAY_CLASS_NAME};
use parse::{expr, tpe, ParseResult, Tokens};

fn parse_array_brackets<'def, 'r>(
    input: Tokens<'def, 'r>,
    tpe: Type<'def>,
    id_gen: &mut IdGen,
) -> ParseResult<'def, 'r, Type<'def>> {
    let (input, _) = match symbol('[')(input) {
        Ok(result) => result,
        Err(_) => return Ok((input, tpe)),
    };

    let (input, size_opt) = if let Ok((input, _)) = symbol(']')(input) {
        (input, None)
    } else {
        let (input, size) = expr::parse(input, id_gen)?;
        let (input, _) = symbol(']')(input)?;
        (input, Some(Box::new(size)))
    };

    let (input, inner) = parse_array_brackets(input, tpe, id_gen)?;

    Ok((
        input,
        Type::Array(ArrayType {
            size_opt,
            underlying: ClassType {
                prefix_opt: None,
                name: NATIVE_ARRAY_CLASS_NAME.to_owned(),
                span_opt: None,
                type_args_opt: Some(vec![inner.clone().to_type_arg()]),
                def_opt: None,
            },
            tpe: Box::new(inner),
        }),
    ))
}

pub fn parse_tail<'def, 'r>(
    input: Tokens<'def, 'r>,
    tpe: Type<'def>,
    id_gen: &mut IdGen,
) -> ParseResult<'def, 'r, Expr<'def>> {
    let (input, tpe) = match parse_array_brackets(input, tpe, id_gen) {
        Ok((input, Type::Array(array))) => (input, array),
        other => return Err(input),
    };
    let (input, initializer_opt) = opt(|i| array_initializer::parse_initializer(i, id_gen))(input)?;

    Ok((
        input,
        Expr::NewArray(NewArray {
            tpe,
            initializer_opt,
        }),
    ))
}

//#[cfg(test)]
//mod tests {
//    use parse::expr::atom;
//    use parse::tree::{
//        ArrayInitializer, ArrayType, ClassType, Expr, Int, Name, NewArray, PrimitiveType,
//        PrimitiveTypeType, Type,
//    };
//    use parse::Tokens;
//    use test_common::{generate_tokens, primitive, span};
//
//    #[test]
//    fn test_array_class() {
//        assert_eq!(
//            atom::parse(&generate_tokens(
//                r#"
//new Test[size]
//            "#
//            )),
//            Ok((
//                &[] as Tokens,
//                Expr::NewArray(NewArray {
//                    tpe: ArrayType {
//                        tpe: Box::new(Type::Class(ClassType {
//                            prefix_opt: None,
//                            name: span(1, 5, "Test"),
//                            type_args_opt: None,
//                            def_opt: None
//                        })),
//                        size_opt: Some(Box::new(Expr::Name(Name {
//                            name: span(1, 10, "size")
//                        })))
//                    },
//                    initializer_opt: None
//                })
//            ))
//        );
//    }
//
//    #[test]
//    fn test_array_primitive() {
//        assert_eq!(
//            atom::parse(&generate_tokens(
//                r#"
//new int[2][]
//            "#
//            )),
//            Ok((
//                &[] as Tokens,
//                Expr::NewArray(NewArray {
//                    tpe: ArrayType {
//                        tpe: Box::new(Type::Array(ArrayType {
//                            tpe: Box::new(Type::Primitive(PrimitiveType {
//                                name: span(1, 5, "int"),
//                                tpe: PrimitiveTypeType::Int
//                            })),
//                            size_opt: None
//                        })),
//                        size_opt: Some(Box::new(Expr::Int(Int {
//                            value: span(1, 9, "2")
//                        })))
//                    },
//                    initializer_opt: None
//                })
//            ))
//        );
//    }
//
//    #[test]
//    fn test_initializer() {
//        assert_eq!(
//            atom::parse(&generate_tokens(
//                r#"
//new int[] { 1, {2}}
//            "#
//            )),
//            Ok((
//                &[] as Tokens,
//                Expr::NewArray(NewArray {
//                    tpe: ArrayType {
//                        tpe: Box::new(Type::Primitive(PrimitiveType {
//                            name: span(1, 5, "int"),
//                            tpe: PrimitiveTypeType::Int
//                        })),
//                        size_opt: None
//                    },
//                    initializer_opt: Some(ArrayInitializer {
//                        items: vec![
//                            Expr::Int(Int {
//                                value: span(1, 13, "1")
//                            }),
//                            Expr::ArrayInitializer(ArrayInitializer {
//                                items: vec![Expr::Int(Int {
//                                    value: span(1, 17, "2")
//                                }),]
//                            })
//                        ]
//                    })
//                })
//            ))
//        );
//    }
//}
