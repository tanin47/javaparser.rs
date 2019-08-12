use parse::combinator::{separated_list, separated_nonempty_list, symbol, word};
use parse::tpe::class;
use parse::tree::{ClassType, Type, TypeArg, WildcardType};
use parse::{ParseResult, Tokens};

pub fn parse_wildcard_extends(input: Tokens) -> ParseResult<Vec<ClassType>> {
    let (input, _) = word("extends")(input)?;

    separated_nonempty_list(symbol("&"), class::parse_no_array)(input)
}

pub fn parse_wildcard_super(input: Tokens) -> ParseResult<ClassType> {
    let (input, _) = word("super")(input)?;

    class::parse_no_array(input)
}

pub fn parse_wildcard(input: Tokens) -> ParseResult<TypeArg> {
    let (input, name) = symbol("?")(input)?;

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

pub fn parse_non_wildcard(original: Tokens) -> ParseResult<TypeArg> {
    let (input, tpe) = class::parse(original)?;

    match tpe {
        Type::Class(tpe) => Ok((input, TypeArg::Class(tpe))),
        Type::Array(tpe) => Ok((input, TypeArg::Array(tpe))),
        _ => Err(original),
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
    let (input, type_args_opt) = match symbol("<")(input) {
        Ok((input, _)) => {
            let (input, type_args) =
                separated_list(symbol(","), parse_wildcard_or_non_wildcard)(input)?;

            let (input, _) = symbol(">")(input)?;
            (input, Some(type_args))
        }
        Err(_) => (input, None),
    };

    Ok((input, type_args_opt))
}

#[cfg(test)]
mod tests {
    use super::parse;
    use parse::tree::{ArrayType, ClassType, Type, TypeArg, WildcardType};
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
                        extends: vec![ClassType {
                            prefix_opt: None,
                            name: span(1, 26, "C"),
                            type_args_opt: None
                        }]
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
<? super C>
            "#
            )),
            Ok((
                &[] as Tokens,
                Some(vec![TypeArg::Wildcard(WildcardType {
                    name: span(1, 2, "?"),
                    super_opt: Some(ClassType {
                        prefix_opt: None,
                        name: span(1, 10, "C"),
                        type_args_opt: None
                    },),
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
<? extends C & S>
            "#
            )),
            Ok((
                &[] as Tokens,
                Some(vec![TypeArg::Wildcard(WildcardType {
                    name: span(1, 2, "?"),
                    super_opt: None,
                    extends: vec![
                        ClassType {
                            prefix_opt: None,
                            name: span(1, 12, "C"),
                            type_args_opt: None
                        },
                        ClassType {
                            prefix_opt: None,
                            name: span(1, 16, "S"),
                            type_args_opt: None
                        },
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
