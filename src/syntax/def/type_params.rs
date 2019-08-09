use nom::bytes::complete::tag;
use nom::multi::{separated_list, separated_nonempty_list};
use nom::IResult;
use syntax::expr::atom::name;
use syntax::tpe::{array, class};
use syntax::tree::{
    ArrayType, Class, ClassType, Expr, Int, PrimitiveType, Span, Type, TypeArg, TypeParam,
};
use syntax::{comment, tpe};

pub fn parse_extends(input: Span) -> IResult<Span, Vec<ClassType>> {
    let (input, _) = comment::parse(input)?;
    match tag("extends")(input) as IResult<Span, Span> {
        Ok((input, _)) => {
            let (input, _) = comment::parse1(input)?;
            separated_nonempty_list(tag("&"), class::parse_no_array)(input)
        }
        Err(_) => Ok((input, vec![])),
    }
}

pub fn parse_type_param(input: Span) -> IResult<Span, TypeParam> {
    let (input, _) = comment::parse(input)?;
    let (input, name) = name::identifier(input)?;
    let (input, extends) = parse_extends(input)?;

    Ok((input, TypeParam { name, extends }))
}

pub fn parse(input: Span) -> IResult<Span, Vec<TypeParam>> {
    let (input, _) = comment::parse(input)?;
    let (input, type_params) = match tag("<")(input) as IResult<Span, Span> {
        Ok((input, _)) => {
            let (input, type_params) = separated_list(tag(","), parse_type_param)(input)?;

            let (input, _) = comment::parse(input)?;
            let (input, _) = tag(">")(input)?;
            (input, type_params)
        }
        Err(_) => (input, vec![]),
    };

    Ok((input, type_params))
}

#[cfg(test)]
mod tests {
    use super::parse;
    use syntax::tree::{
        ArrayType, ClassType, Expr, Int, Method, PrimitiveType, ReturnStmt, Type, TypeArg,
        TypeParam,
    };
    use test_common::{code, span};

    #[test]
    fn test() {
        assert_eq!(
            parse(code(
                r#"
<A, B extends A, C extends String & Another<A>>
            "#
                .trim()
            )),
            Ok((
                span(1, 48, ""),
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
        assert_eq!(parse(code("")), Ok((span(1, 1, ""), vec![])));
    }
}
