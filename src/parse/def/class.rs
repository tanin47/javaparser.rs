use parse::combinator::{identifier, separated_nonempty_list, symbol, word};
use parse::def::{class_body, type_params};
use parse::tpe::class;
use parse::tree::{Class, ClassBody, ClassType, Modifier};
use parse::{ParseResult, Tokens};
use tokenize::span::Span;

pub fn parse_implements(input: Tokens) -> ParseResult<Vec<ClassType>> {
    if let Ok((input, _)) = word("implements")(input) {
        let (input, classes) = separated_nonempty_list(symbol(','), class::parse_no_array)(input)?;
        Ok((input, classes))
    } else {
        Ok((input, vec![]))
    }
}

fn parse_extend(input: Tokens) -> ParseResult<Option<ClassType>> {
    if let Ok((input, _)) = word("extends")(input) {
        let (input, class) = class::parse_no_array(input)?;
        Ok((input, Some(class)))
    } else {
        Ok((input, None))
    }
}

pub fn parse_tail<'a>(
    input: Tokens<'a>,
    modifiers: Vec<Modifier<'a>>,
) -> ParseResult<'a, Class<'a>> {
    let (input, name) = identifier(input)?;
    let (input, type_params) = type_params::parse(input)?;
    let (input, extend_opt) = parse_extend(input)?;
    let (input, implements) = parse_implements(input)?;
    println!("{:#?}", &input[0]);

    let (input, body) = class_body::parse(input)?;

    Ok((
        input,
        Class {
            modifiers,
            name,
            type_params,
            extend_opt,
            implements,
            body: ClassBody { items: vec![] },
        },
    ))
}

pub fn parse_prefix(input: Tokens) -> ParseResult<Span> {
    word("class")(input)
}

#[cfg(test)]
mod tests {
    use parse::tree::{
        Annotated, Class, ClassBody, ClassType, CompilationUnitItem, Keyword, MarkerAnnotated,
        Modifier, TypeArg, TypeParam,
    };
    use parse::{compilation_unit, Tokens};
    use test_common::{code, primitive, span};

    #[test]
    fn test_bare() {
        assert_eq!(
            compilation_unit::parse_item(&code(
                r#"
@Anno private class Test extends Super {}
            "#
            )),
            Ok((
                &[] as Tokens,
                CompilationUnitItem::Class(Class {
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
                })
            ))
        );
    }

    #[test]
    fn test_type_params() {
        assert_eq!(
            compilation_unit::parse_item(&code(
                r#"
class Test<A> implements Super, Super2<A> {}
            "#
            )),
            Ok((
                &[] as Tokens,
                CompilationUnitItem::Class(Class {
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
                })
            ))
        );
    }
}
