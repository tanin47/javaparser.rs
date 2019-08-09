use nom::character::complete::multispace0;
use nom::character::is_space;
use nom::IResult;

use nom::branch::alt;
use nom::multi::{many0, separated_list, separated_nonempty_list};
use syntax::def::{annotateds, class_body, modifiers, type_params};
use syntax::expr::atom::name;
use syntax::tpe::class;
use syntax::tree::{Class, ClassType, Modifier};
use syntax::tree::{Interface, Span};
use syntax::{comment, tag};

fn parse_extends(input: Span) -> IResult<Span, Vec<ClassType>> {
    let (input, _) = comment::parse(input)?;
    match tag("extends")(input) as IResult<Span, Span> {
        Ok((input, _)) => {
            let (input, classes) = separated_nonempty_list(tag(","), class::parse_no_array)(input)?;
            Ok((input, classes))
        }
        Err(_) => Ok((input, vec![])),
    }
}

pub fn parse_tail<'a>(
    input: Span<'a>,
    modifiers: Vec<Modifier<'a>>,
) -> IResult<Span<'a>, Interface<'a>> {
    let (input, name) = name::identifier(input)?;
    let (input, type_params) = type_params::parse(input)?;

    let (input, extends) = parse_extends(input)?;

    let (input, body) = class_body::parse(input)?;

    Ok((
        input,
        Interface {
            modifiers,
            name,
            type_params,
            extends,
            body,
        },
    ))
}

pub fn parse_prefix(input: Span) -> IResult<Span, Span> {
    tag("interface")(input)
}

pub fn parse(input: Span) -> IResult<Span, Interface> {
    let (input, modifiers) = modifiers::parse(input)?;
    let (input, _) = parse_prefix(input)?;
    parse_tail(input, modifiers)
}

#[cfg(test)]
mod tests {
    use super::parse;
    use syntax::tree::{
        Annotated, Block, Class, ClassBody, ClassType, Expr, Int, Interface, Keyword,
        MarkerAnnotated, Method, Modifier, Param, PrimitiveType, ReturnStmt, Statement, Type,
        TypeArg, TypeParam,
    };
    use test_common::{code, primitive, span};

    #[test]
    fn test_bare() {
        assert_eq!(
            parse(code(
                r#"
@Anno private interface Test {}
            "#
                .trim()
            )),
            Ok((
                span(1, 32, ""),
                Interface {
                    modifiers: vec![
                        Modifier::Annotated(Annotated::Marker(MarkerAnnotated {
                            name: span(1, 2, "Anno")
                        })),
                        Modifier::Keyword(Keyword {
                            name: span(1, 7, "private")
                        })
                    ],
                    name: span(1, 25, "Test"),
                    type_params: vec![],
                    extends: vec![],
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
interface Test<A> extends Super, Super2<A> {}
            "#
                .trim()
            )),
            Ok((
                span(1, 46, ""),
                Interface {
                    modifiers: vec![],
                    name: span(1, 11, "Test"),
                    type_params: vec![TypeParam {
                        name: span(1, 16, "A"),
                        extends: vec![]
                    }],
                    extends: vec![
                        ClassType {
                            prefix_opt: None,
                            name: span(1, 27, "Super"),
                            type_args_opt: None
                        },
                        ClassType {
                            prefix_opt: None,
                            name: span(1, 34, "Super2"),
                            type_args_opt: Some(vec![TypeArg::Class(ClassType {
                                prefix_opt: None,
                                name: span(1, 41, "A"),
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
