use nom::bytes::complete::{tag, take, take_till, take_while};
use nom::character::complete::multispace0;
use nom::character::is_space;
use nom::IResult;

use nom::branch::alt;
use nom::multi::{many0, separated_list};
use syntax::def::{annotateds, param, type_params};
use syntax::expr::atom::name;
use syntax::statement::block;
use syntax::tree::{Class, Method};
use syntax::tree::{Constructor, Span};
use syntax::{comment, tpe};

fn modifier(input: Span) -> IResult<Span, Span> {
    alt((tag("private"), tag("protected"), tag("public")))(input)
}

fn modifiers(input: Span) -> IResult<Span, Vec<Span>> {
    let (input, _) = comment::parse(input)?;
    separated_list(comment::parse, modifier)(input)
}

pub fn parse(input: Span) -> IResult<Span, Constructor> {
    let (input, annotateds) = annotateds::parse(input)?;
    let (input, modifiers) = modifiers(input)?;
    let (input, type_params) = type_params::parse(input)?;

    let (input, name) = name::identifier(input)?;

    let (input, _) = comment::parse(input)?;
    let (input, _) = tag("(")(input)?;

    let (input, params) = separated_list(tag(","), param::parse)(input)?;

    let (input, _) = comment::parse(input)?;
    let (input, _) = tag(")")(input)?;

    let (input, block) = block::parse_block(input)?;

    Ok((
        input,
        Constructor {
            annotateds,
            modifiers,
            type_params,
            name,
            params,
            block,
        },
    ))
}

#[cfg(test)]
mod tests {
    use super::parse;
    use syntax::tree::{
        Annotated, Block, ClassType, Constructor, Expr, Int, MarkerAnnotated, Method, Param,
        PrimitiveType, ReturnStmt, Statement, Type, TypeArg, TypeParam,
    };
    use test_common::{code, primitive, span};

    #[test]
    fn test_constructor() {
        assert_eq!(
            parse(code(
                r#"
@Anno private constructor() {}
            "#
                .trim()
            )),
            Ok((
                span(1, 31, ""),
                Constructor {
                    annotateds: vec![Annotated::Marker(MarkerAnnotated {
                        name: span(1, 2, "Anno")
                    })],
                    name: span(1, 15, "constructor"),
                    modifiers: vec![span(1, 7, "private")],
                    type_params: vec![],
                    params: vec![],
                    block: Block { stmts: vec![] },
                }
            ))
        );
    }

    #[test]
    fn test_constructor_with_params() {
        assert_eq!(
            parse(code(
                r#"
<A> con(Test t, A a) {}
            "#
                .trim()
            )),
            Ok((
                span(1, 24, ""),
                Constructor {
                    annotateds: vec![],
                    name: span(1, 5, "con"),
                    modifiers: vec![],
                    type_params: vec![TypeParam {
                        name: span(1, 2, "A"),
                        extends: vec![],
                    }],
                    params: vec![
                        Param {
                            modifiers: vec![],
                            tpe: Type::Class(ClassType {
                                prefix_opt: None,
                                name: span(1, 9, "Test"),
                                type_args_opt: None
                            }),
                            is_varargs: false,
                            name: span(1, 14, "t"),
                        },
                        Param {
                            modifiers: vec![],
                            tpe: Type::Class(ClassType {
                                prefix_opt: None,
                                name: span(1, 17, "A"),
                                type_args_opt: None
                            }),
                            is_varargs: false,
                            name: span(1, 19, "a"),
                        }
                    ],
                    block: Block { stmts: vec![] },
                }
            ))
        );
    }
}
