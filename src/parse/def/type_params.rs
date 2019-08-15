use parse::combinator::{identifier, keyword, separated_list, separated_nonempty_list, symbol};
use parse::tpe::class;
use parse::tree::{ClassType, TypeParam};
use parse::{ParseResult, Tokens};

pub fn parse_extends(input: Tokens) -> ParseResult<Vec<ClassType>> {
    if let Ok((input, _)) = keyword("extends")(input) {
        separated_nonempty_list(symbol('&'), class::parse_no_array)(input)
    } else {
        Ok((input, vec![]))
    }
}

pub fn parse_type_param(input: Tokens) -> ParseResult<TypeParam> {
    let (input, name) = identifier(input)?;
    let (input, extends) = parse_extends(input)?;

    Ok((input, TypeParam { name, extends }))
}

pub fn parse(input: Tokens) -> ParseResult<Vec<TypeParam>> {
    if let Ok((input, _)) = symbol('<')(input) {
        let (input, type_params) = separated_list(symbol(','), parse_type_param)(input)?;
        let (input, _) = symbol('>')(input)?;
        Ok((input, type_params))
    } else {
        Ok((input, vec![]))
    }
}

#[cfg(test)]
mod tests {
    use super::parse;
    use parse::tree::{ClassType, TypeArg, TypeParam};
    use parse::Tokens;
    use test_common::{code, span};

    #[test]
    fn test() {
        assert_eq!(
            parse(&code(
                r#"
<A, B extends A, C extends String & Another<A>>
            "#
            )),
            Ok((
                &[] as Tokens,
                vec![
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
                    },
                    TypeParam {
                        name: span(1, 18, "C"),
                        extends: vec![
                            ClassType {
                                prefix_opt: None,
                                name: span(1, 28, "String"),
                                type_args_opt: None
                            },
                            ClassType {
                                prefix_opt: None,
                                name: span(1, 37, "Another"),
                                type_args_opt: Some(vec![TypeArg::Class(ClassType {
                                    prefix_opt: None,
                                    name: span(1, 45, "A"),
                                    type_args_opt: None
                                })])
                            }
                        ]
                    },
                ]
            ))
        );
    }

    #[test]
    fn test_empty() {
        assert_eq!(parse(&code("")), Ok((&[] as Tokens, vec![])));
    }
}
