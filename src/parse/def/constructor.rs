use parse::combinator::{opt, separated_list, symbol};
use parse::def::method::parse_throws;
use parse::def::param;
use parse::statement::block;
use parse::tree::{Constructor, Modifier, TypeParam};
use parse::{ParseResult, Tokens};
use tokenize::span::Span;

pub fn parse<'a>(
    input: Tokens<'a>,
    modifiers: Vec<Modifier<'a>>,
    type_params: Vec<TypeParam<'a>>,
    name: Span<'a>,
) -> ParseResult<'a, Constructor<'a>> {
    let (input, _) = symbol('(')(input)?;
    let (input, params) = separated_list(symbol(','), param::parse)(input)?;
    let (input, _) = symbol(')')(input)?;
    let (input, throws) = parse_throws(input)?;

    let (input, block) = block::parse_block(input)?;
    let (input, _) = opt(symbol(';'))(input)?;

    Ok((
        input,
        Constructor {
            modifiers,
            type_params,
            name,
            params,
            throws,
            block,
        },
    ))
}

#[cfg(test)]
mod tests {
    use parse::def::class_body;
    use parse::tree::{
        Annotated, Block, ClassBodyItem, ClassType, Constructor, Keyword, MarkerAnnotated,
        Modifier, Param, Type, TypeParam,
    };
    use parse::Tokens;
    use test_common::{code, primitive, span};

    #[test]
    fn test_constructor() {
        assert_eq!(
            class_body::parse_item(&code(
                r#"
@Anno private constructor() throws Exp {}
            "#
            )),
            Ok((
                &[] as Tokens,
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
                    throws: vec![ClassType {
                        prefix_opt: None,
                        name: span(1, 36, "Exp"),
                        type_args_opt: None
                    }],
                    block: Block { stmts: vec![] },
                })
            ))
        );
    }

    #[test]
    fn test_constructor_with_params() {
        assert_eq!(
            class_body::parse_item(&code(
                r#"
<A> con(Test t, A a) {}
            "#
            )),
            Ok((
                &[] as Tokens,
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
                    throws: vec![],
                    block: Block { stmts: vec![] },
                })
            ))
        );
    }
}
