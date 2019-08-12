use parse::combinator::{get_and_followed_by, identifier, is_not, symbol};
use parse::tpe::{array, type_args};
use parse::tree::{ClassType, Type};
use parse::{ParseResult, Tokens};
use tokenize::span::Span;

pub fn contains_type_args(class: &ClassType) -> bool {
    if class.type_args_opt.is_some() {
        true
    } else {
        match &class.prefix_opt {
            Some(prefix) => contains_type_args(prefix),
            None => false,
        }
    }
}

pub fn parse_tail<'a>(
    input: Tokens<'a>,
    name: Span<'a>,
    prefix_opt: Option<ClassType<'a>>,
) -> ParseResult<'a, ClassType<'a>> {
    let (input, type_args_opt) = type_args::parse(input)?;

    let tpe = ClassType {
        prefix_opt: match prefix_opt {
            Some(prefix) => Some(Box::new(prefix)),
            None => None,
        },
        name,
        type_args_opt,
    };

    if let Ok((input, _)) = get_and_followed_by(symbol("."), is_not(symbol(".")))(input) {
        parse_no_array_with_prefix(input, Some(tpe))
    } else {
        Ok((input, tpe))
    }
}

fn parse_no_array_with_prefix<'a>(
    input: Tokens<'a>,
    prefix_opt: Option<ClassType<'a>>,
) -> ParseResult<'a, ClassType<'a>> {
    let (input, name) = identifier(input)?;

    parse_tail(input, name, prefix_opt)
}

pub fn parse_no_array(input: Tokens) -> ParseResult<ClassType> {
    parse_no_array_with_prefix(input, None)
}

pub fn parse(input: Tokens) -> ParseResult<Type> {
    let (input, tpe) = parse_no_array(input)?;
    array::parse_tail(input, Type::Class(tpe))
}

#[cfg(test)]
mod tests {
    use super::parse;
    use parse::tree::{ArrayType, ClassType, Type, TypeArg};
    use parse::Tokens;
    use test_common::{code, span};

    #[test]
    fn test_simple() {
        assert_eq!(
            parse(&code(
                r#"
Test
            "#
            )),
            Ok((
                &[] as Tokens,
                Type::Class(ClassType {
                    prefix_opt: None,
                    name: span(1, 1, "Test"),
                    type_args_opt: None
                })
            ))
        );
    }

    #[test]
    fn test_chain() {
        assert_eq!(
            parse(&code(
                r#"
Parent<A>.Test
            "#
            )),
            Ok((
                &[] as Tokens,
                Type::Class(ClassType {
                    prefix_opt: Some(Box::new(ClassType {
                        prefix_opt: None,
                        name: span(1, 1, "Parent"),
                        type_args_opt: Some(vec![TypeArg::Class(ClassType {
                            prefix_opt: None,
                            name: span(1, 8, "A"),
                            type_args_opt: None
                        })])
                    })),
                    name: span(1, 11, "Test"),
                    type_args_opt: None
                })
            ))
        );
    }

    #[test]
    fn test_class() {
        assert_eq!(
            parse(&code(
                r#"
Test<Another<A>, T[]>
            "#
            )),
            Ok((
                &[] as Tokens,
                Type::Class(ClassType {
                    prefix_opt: None,
                    name: span(1, 1, "Test"),
                    type_args_opt: Some(vec![
                        TypeArg::Class(ClassType {
                            prefix_opt: None,
                            name: span(1, 6, "Another"),
                            type_args_opt: Some(vec![TypeArg::Class(ClassType {
                                prefix_opt: None,
                                name: span(1, 14, "A"),
                                type_args_opt: None
                            })])
                        }),
                        TypeArg::Array(ArrayType {
                            tpe: Box::new(Type::Class(ClassType {
                                prefix_opt: None,
                                name: span(1, 18, "T"),
                                type_args_opt: None
                            })),
                            size_opt: None
                        })
                    ])
                })
            ))
        );
    }

    #[test]
    fn test_array() {
        assert_eq!(
            parse(&code(
                r#"
Test[]
            "#
            )),
            Ok((
                &[] as Tokens,
                Type::Array(ArrayType {
                    tpe: Box::new(Type::Class(ClassType {
                        prefix_opt: None,
                        name: span(1, 1, "Test"),
                        type_args_opt: None
                    })),
                    size_opt: None
                })
            ))
        );
    }

    #[test]
    fn test_array_3d() {
        assert_eq!(
            parse(&code(
                r#"
Test[][][]
            "#
            )),
            Ok((
                &[] as Tokens,
                Type::Array(ArrayType {
                    tpe: Box::new(Type::Array(ArrayType {
                        tpe: Box::new(Type::Array(ArrayType {
                            tpe: Box::new(Type::Class(ClassType {
                                prefix_opt: None,
                                name: span(1, 1, "Test"),
                                type_args_opt: None
                            })),
                            size_opt: None
                        })),
                        size_opt: None
                    })),
                    size_opt: None
                })
            ))
        );
    }
}
