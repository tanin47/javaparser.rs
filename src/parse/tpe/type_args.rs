use parse::combinator::{keyword, separated_list, separated_nonempty_list, symbol};
use parse::tpe::{class, primitive, reference};
use parse::tree::{ClassType, ReferenceType, Type, TypeArg, WildcardType};
use parse::{ParseResult, Tokens};

pub fn parse_wildcard_extends(input: Tokens) -> ParseResult<Vec<ReferenceType>> {
    let (input, _) = keyword("extends")(input)?;

    separated_nonempty_list(symbol('&'), reference::parse)(input)
}

pub fn parse_wildcard_super(input: Tokens) -> ParseResult<ReferenceType> {
    let (input, _) = keyword("super")(input)?;

    reference::parse(input)
}

pub fn parse_wildcard(input: Tokens) -> ParseResult<TypeArg> {
    let (input, name) = symbol('?')(input)?;

    let (input, extends, super_opt) = match parse_wildcard_extends(input) {
        Ok((input, extends)) => (input, extends, None),
        Err(_) => {
            let (input, super_opt) = match parse_wildcard_super(input) {
                Ok((input, sup)) => (input, Some(sup)),
                Err(_) => (input, None),
            };
            (input, vec![], super_opt)
        }
    };

    Ok((
        input,
        TypeArg::Wildcard(WildcardType {
            name,
            super_opt,
            extends,
        }),
    ))
}

pub fn parse_non_wildcard(input: Tokens) -> ParseResult<TypeArg> {
    let (input, tpe) = if let Ok(ok) = primitive::parse(input) {
        ok
    } else if let Ok(ok) = class::parse(input) {
        ok
    } else {
        return Err(input);
    };

    match tpe {
        Type::Class(tpe) => Ok((input, TypeArg::Class(tpe))),
        Type::Array(tpe) => Ok((input, TypeArg::Array(tpe))),
        _ => Err(input),
    }
}

pub fn parse_wildcard_or_non_wildcard(input: Tokens) -> ParseResult<TypeArg> {
    if let Ok((input, type_arg)) = parse_wildcard(input) {
        Ok((input, type_arg))
    } else if let Ok((input, type_arg)) = parse_non_wildcard(input) {
        Ok((input, type_arg))
    } else {
        Err(input)
    }
}

pub fn parse(input: Tokens) -> ParseResult<Option<Vec<TypeArg>>> {
    let (input, type_args_opt) = match symbol('<')(input) {
        Ok((input, _)) => {
            let (input, type_args) =
                separated_list(symbol(','), parse_wildcard_or_non_wildcard)(input)?;

            let (input, _) = symbol('>')(input)?;
            (input, Some(type_args))
        }
        Err(_) => (input, None),
    };

    Ok((input, type_args_opt))
}

#[cfg(test)]
mod tests {
    use super::parse;
    use parse::tree::{
        ArrayType, ClassType, PrimitiveType, ReferenceType, Type, TypeArg, WildcardType,
    };
    use parse::Tokens;
    use test_common::{code, span};

    #[test]
    fn test() {
        assert_eq!(
            parse(&code(
                r#"
<Test<A[]>, B, ? extends C>
            "#
            )),
            Ok((
                &[] as Tokens,
                Some(vec![
                    TypeArg::Class(ClassType {
                        prefix_opt: None,
                        name: span(1, 2, "Test"),
                        type_args_opt: Some(vec![TypeArg::Array(ArrayType {
                            tpe: Box::new(Type::Class(ClassType {
                                prefix_opt: None,
                                name: span(1, 7, "A"),
                                type_args_opt: None
                            })),
                            size_opt: None
                        })])
                    }),
                    TypeArg::Class(ClassType {
                        prefix_opt: None,
                        name: span(1, 13, "B"),
                        type_args_opt: None
                    }),
                    TypeArg::Wildcard(WildcardType {
                        name: span(1, 16, "?"),
                        super_opt: None,
                        extends: vec![ReferenceType::Class(ClassType {
                            prefix_opt: None,
                            name: span(1, 26, "C"),
                            type_args_opt: None
                        })]
                    })
                ])
            ))
        );
    }

    #[test]
    fn test_wildcard_super() {
        assert_eq!(
            parse(&code(
                r#"
<? super int[]>
            "#
            )),
            Ok((
                &[] as Tokens,
                Some(vec![TypeArg::Wildcard(WildcardType {
                    name: span(1, 2, "?"),
                    super_opt: Some(ReferenceType::Array(ArrayType {
                        tpe: Box::new(Type::Primitive(PrimitiveType {
                            name: span(1, 10, "int")
                        })),
                        size_opt: None
                    })),
                    extends: vec![]
                })])
            ))
        );
    }

    #[test]
    fn test_wildcard_extends() {
        assert_eq!(
            parse(&code(
                r#"
<? extends boolean[] & S>
            "#
            )),
            Ok((
                &[] as Tokens,
                Some(vec![TypeArg::Wildcard(WildcardType {
                    name: span(1, 2, "?"),
                    super_opt: None,
                    extends: vec![
                        ReferenceType::Array(ArrayType {
                            tpe: Box::new(Type::Primitive(PrimitiveType {
                                name: span(1, 12, "boolean")
                            })),
                            size_opt: None
                        }),
                        ReferenceType::Class(ClassType {
                            prefix_opt: None,
                            name: span(1, 24, "S"),
                            type_args_opt: None
                        }),
                    ]
                })])
            ))
        );
    }

    #[test]
    fn test_array_2d() {
        assert_eq!(parse(&code("")), Ok((&[] as Tokens, None)));
    }
}
