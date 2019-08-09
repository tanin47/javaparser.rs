use nom::IResult;

use nom::branch::alt;
use nom::multi::{many0, separated_list, separated_nonempty_list};
use syntax::def::{annotateds, class_body, modifiers, type_params};
use syntax::expr::atom::name;
use syntax::tpe::class;
use syntax::tree::{Class, ClassType};
use syntax::tree::{Modifier, Span};
use syntax::{comment, tag};

pub fn parse_implements(input: Span) -> IResult<Span, Vec<ClassType>> {
    match tag("implements")(input) as IResult<Span, Span> {
        Ok((input, _)) => {
            let (input, classes) = separated_nonempty_list(tag(","), class::parse_no_array)(input)?;
            Ok((input, classes))
        }
        Err(_) => Ok((input, vec![])),
    }
}

fn parse_extend(input: Span) -> IResult<Span, Option<ClassType>> {
    match tag("extends")(input) as IResult<Span, Span> {
        Ok((input, _)) => {
            let (input, class) = class::parse_no_array(input)?;
            Ok((input, Some(class)))
        }
        Err(_) => Ok((input, None)),
    }
}

pub fn parse_tail<'a>(
    input: Span<'a>,
    modifiers: Vec<Modifier<'a>>,
) -> IResult<Span<'a>, Class<'a>> {
    let (input, name) = name::identifier(input)?;
    let (input, type_params) = type_params::parse(input)?;

    let (input, extend_opt) = parse_extend(input)?;
    let (input, implements) = parse_implements(input)?;

    let (input, body) = class_body::parse(input)?;

    Ok((
        input,
        Class {
            modifiers,
            name,
            type_params,
            extend_opt,
            implements,
            body,
        },
    ))
}

pub fn parse_prefix(input: Span) -> IResult<Span, Span> {
    tag("class")(input)
}

pub fn parse(input: Span) -> IResult<Span, Class> {
    let (input, modifiers) = modifiers::parse(input)?;
    let (input, _) = parse_prefix(input)?;
    parse_tail(input, modifiers)
}

#[cfg(test)]
mod tests {
    use super::parse;
    use syntax::tree::{
        Annotated, Block, Class, ClassBody, ClassType, Expr, Int, Keyword, MarkerAnnotated, Method,
        Modifier, Param, PrimitiveType, ReturnStmt, Statement, Type, TypeArg, TypeParam,
    };
    use test_common::{code, primitive, span};

    #[test]
    fn test_bare() {
        assert_eq!(
            parse(code(
                r#"
@Anno private class Test extends Super {}
            "#
                .trim()
            )),
            Ok((
                span(1, 42, ""),
                Class {
                    modifiers: vec![
                        Modifier::Annotated(Annotated::Marker(MarkerAnnotated {
                            name: span(1, 2, "Anno")
                        })),
                        Modifier::Keyword(Keyword {
                            name: span(1, 7, "private")
                        })
                    ],
                    name: span(1, 21, "Test"),
                    type_params: vec![],
                    extend_opt: Some(ClassType {
                        prefix_opt: None,
                        name: span(1, 34, "Super"),
                        type_args_opt: None
                    }),
                    implements: vec![],
                    body: ClassBody { items: vec![] }
                }
            ))
        );
    }

    #[test]
    fn test_type_params() {
        assert_eq!(
            parse(code(
                r#"
class Test<A> implements Super, Super2<A> {}
            "#
                .trim()
            )),
            Ok((
                span(1, 45, ""),
                Class {
                    modifiers: vec![],
                    name: span(1, 7, "Test"),
                    type_params: vec![TypeParam {
                        name: span(1, 12, "A"),
                        extends: vec![]
                    }],
                    extend_opt: None,
                    implements: vec![
                        ClassType {
                            prefix_opt: None,
                            name: span(1, 26, "Super"),
                            type_args_opt: None
                        },
                        ClassType {
                            prefix_opt: None,
                            name: span(1, 33, "Super2"),
                            type_args_opt: Some(vec![TypeArg::Class(ClassType {
                                prefix_opt: None,
                                name: span(1, 40, "A"),
                                type_args_opt: None
                            })])
                        },
                    ],
                    body: ClassBody { items: vec![] }
                }
            ))
        );
    }
}
