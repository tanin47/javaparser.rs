use nom::character::complete::multispace0;
use nom::character::is_space;
use nom::IResult;

use nom::branch::alt;
use nom::multi::{many0, separated_list};
use syntax::def::{annotateds, param, type_params};
use syntax::expr::atom::name;
use syntax::statement::block;
use syntax::tree::{Class, Method, Modifier, TypeParam};
use syntax::tree::{Constructor, Span};
use syntax::{comment, tag, tpe};

pub fn parse<'a>(
    input: Span<'a>,
    modifiers: Vec<Modifier<'a>>,
    type_params: Vec<TypeParam<'a>>,
    name: Span<'a>,
) -> IResult<Span<'a>, Constructor<'a>> {
    let (input, _) = tag("(")(input)?;
    let (input, params) = separated_list(tag(","), param::parse)(input)?;
    let (input, _) = tag(")")(input)?;

    let (input, block) = block::parse_block(input)?;

    Ok((
        input,
        Constructor {
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
    use syntax::def::class_body;
    use syntax::tree::{
        Annotated, Block, ClassBodyItem, ClassType, Constructor, Expr, Int, Keyword,
        MarkerAnnotated, Method, Modifier, Param, PrimitiveType, ReturnStmt, Statement, Type,
        TypeArg, TypeParam,
    };
    use test_common::{code, primitive, span};

    #[test]
    fn test_constructor() {
        assert_eq!(
            class_body::parse_item(code(
                r#"
@Anno private constructor() {}
            "#
                .trim()
            )),
            Ok((
                span(1, 31, ""),
                ClassBodyItem::Constructor(Constructor {
                    modifiers: vec![
                        Modifier::Annotated(Annotated::Marker(MarkerAnnotated {
                            name: span(1, 2, "Anno")
                        })),
                        Modifier::Keyword(Keyword {
                            name: span(1, 7, "private")
                        })
                    ],
                    name: span(1, 15, "constructor"),
                    type_params: vec![],
                    params: vec![],
                    block: Block { stmts: vec![] },
                })
            ))
        );
    }

    #[test]
    fn test_constructor_with_params() {
        assert_eq!(
            class_body::parse_item(code(
                r#"
<A> con(Test t, A a) {}
            "#
                .trim()
            )),
            Ok((
                span(1, 24, ""),
                ClassBodyItem::Constructor(Constructor {
                    modifiers: vec![],
                    name: span(1, 5, "con"),
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
                })
            ))
        );
    }
}
