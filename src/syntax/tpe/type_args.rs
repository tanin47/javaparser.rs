use nom::branch::alt;
use nom::character::is_digit;
use nom::error::ErrorKind;
use nom::multi::{separated_list, separated_nonempty_list, separated_nonempty_listc};
use nom::IResult;
use syntax::tpe::{array, class};
use syntax::tree::{
    ArrayType, Class, ClassType, Expr, Int, PrimitiveType, Span, Type, TypeArg, WildcardType,
};
use syntax::{comment, tag, tpe};

pub fn parse_wildcard_extends(input: Span) -> IResult<Span, Vec<ClassType>> {
    let (input, _) = comment::parse(input)?;
    let (input, _) = tag("extends")(input)?;
    let (input, _) = comment::parse1(input)?;

    separated_nonempty_list(tag("&"), class::parse_no_array)(input)
}

pub fn parse_wildcard_super(input: Span) -> IResult<Span, ClassType> {
    let (input, _) = comment::parse(input)?;
    let (input, _) = tag("super")(input)?;
    let (input, _) = comment::parse1(input)?;

    class::parse_no_array(input)
}

pub fn parse_wildcard(input: Span) -> IResult<Span, TypeArg> {
    let (input, _) = comment::parse(input)?;
    let (input, name) = tag("?")(input)?;

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

pub fn parse_non_wildcard(input: Span) -> IResult<Span, TypeArg> {
    let (input, tpe) = class::parse(input)?;

    match tpe {
        Type::Class(tpe) => Ok((input, TypeArg::Class(tpe))),
        Type::Array(tpe) => Ok((input, TypeArg::Array(tpe))),
        _ => Err(nom::Err::Failure((input, ErrorKind::Tag))),
    }
}

pub fn parse(input: Span) -> IResult<Span, Option<Vec<TypeArg>>> {
    let (input, _) = comment::parse(input)?;
    let (input, type_args_opt) = match tag("<")(input) {
        Ok((input, _)) => {
            let (input, type_args) =
                separated_list(tag(","), alt((parse_wildcard, parse_non_wildcard)))(input)?;

            let (input, _) = comment::parse(input)?;
            let (input, _) = tag(">")(input)?;
            (input, Some(type_args))
        }
        Err(_) => (input, None),
    };

    Ok((input, type_args_opt))
}

#[cfg(test)]
mod tests {
    use super::parse;
    use syntax::tree::{
        ArrayType, ClassType, Expr, Int, Method, PrimitiveType, ReturnStmt, Type, TypeArg,
        WildcardType,
    };
    use test_common::{code, span};

    #[test]
    fn test() {
        assert_eq!(
            parse(code(
                r#"
<Test<A[]>, B, ? extends C>
            "#
                .trim()
            )),
            Ok((
                span(1, 28, ""),
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
            parse(code(
                r#"
<? super C>
            "#
                .trim()
            )),
            Ok((
                span(1, 12, ""),
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
            parse(code(
                r#"
<? extends C & S>
            "#
                .trim()
            )),
            Ok((
                span(1, 18, ""),
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
        assert_eq!(parse(code("")), Ok((span(1, 1, ""), None)));
    }
}
