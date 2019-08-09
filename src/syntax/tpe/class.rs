use nom::bytes::complete::is_not;
use nom::IResult;
use syntax::expr::atom::name;
use syntax::tpe::{array, type_args};
use syntax::tree::{ArrayType, Class, ClassType, Expr, Int, PrimitiveType, Span, Type};
use syntax::{comment, tag, tag_and_followed_by};

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
    input: Span<'a>,
    name: Span<'a>,
    prefix_opt: Option<ClassType<'a>>,
) -> IResult<Span<'a>, ClassType<'a>> {
    let (input, type_args_opt) = type_args::parse(input)?;

    let tpe = ClassType {
        prefix_opt: match prefix_opt {
            Some(prefix) => Some(Box::new(prefix)),
            None => None,
        },
        name,
        type_args_opt,
    };

    if let Ok((input, _)) = tag_and_followed_by(".", is_not("."))(input) {
        parse_no_array_with_prefix(input, Some(tpe))
    } else {
        Ok((input, tpe))
    }
}

fn parse_no_array_with_prefix<'a>(
    input: Span<'a>,
    prefix_opt: Option<ClassType<'a>>,
) -> IResult<Span<'a>, ClassType<'a>> {
    let (input, _) = comment::parse(input)?;
    let (input, name) = name::identifier(input)?;

    parse_tail(input, name, prefix_opt)
}

pub fn parse_no_array(input: Span) -> IResult<Span, ClassType> {
    parse_no_array_with_prefix(input, None)
}

pub fn parse(input: Span) -> IResult<Span, Type> {
    let (input, tpe) = parse_no_array(input)?;
    array::parse_tail(input, Type::Class(tpe))
}

#[cfg(test)]
mod tests {
    use super::parse;
    use syntax::tree::{
        ArrayType, ClassType, Expr, Int, Method, PrimitiveType, ReturnStmt, Type, TypeArg,
    };
    use test_common::{code, span};

    #[test]
    fn test_simple() {
        assert_eq!(
            parse(code(
                r#"
Test
            "#
                .trim()
            )),
            Ok((
                span(1, 5, ""),
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
            parse(code(
                r#"
Parent<A>.Test
            "#
                .trim()
            )),
            Ok((
                span(1, 15, ""),
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
            parse(code(
                r#"
Test<Another<A>, T[]>
            "#
                .trim()
            )),
            Ok((
                span(1, 22, ""),
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
            parse(code(
                r#"
Test[]
            "#
                .trim()
            )),
            Ok((
                span(1, 7, ""),
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
            parse(code(
                r#"
Test[][][]
            "#
                .trim()
            )),
            Ok((
                span(1, 11, ""),
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
