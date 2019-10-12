use parse::combinator::{identifier, keyword, opt, separated_nonempty_list, symbol};
use parse::def::{class_body, type_params};
use parse::id_gen::IdGen;
use parse::tpe::class;
use parse::tree::{Class, ClassBody, ClassType, Modifier};
use parse::{ParseResult, Tokens};
use std::cell::RefCell;
use tokenize::span::Span;

pub fn parse_implements<'def, 'r>(
    input: Tokens<'def, 'r>,
) -> ParseResult<'def, 'r, Vec<ClassType<'def>>> {
    if let Ok((input, _)) = keyword("implements")(input) {
        let (input, classes) = separated_nonempty_list(symbol(','), class::parse_no_array)(input)?;
        Ok((input, classes))
    } else {
        Ok((input, vec![]))
    }
}

fn parse_extend<'def, 'r>(
    input: Tokens<'def, 'r>,
) -> ParseResult<'def, 'r, Option<ClassType<'def>>> {
    if let Ok((input, _)) = keyword("extends")(input) {
        let (input, class) = class::parse_no_array(input)?;
        Ok((input, Some(class)))
    } else {
        Ok((input, None))
    }
}

pub fn parse_tail<'def, 'r>(
    input: Tokens<'def, 'r>,
    modifiers: Vec<Modifier<'def>>,
    id_gen: &mut IdGen,
) -> ParseResult<'def, 'r, Class<'def>> {
    let (input, name) = identifier(input)?;
    let (input, type_params) = type_params::parse(input)?;
    let (input, extend_opt) = parse_extend(input)?;
    let (input, implements) = parse_implements(input)?;

    let (input, body) = class_body::parse(input, id_gen)?;
    let (input, _) = opt(symbol(';'))(input)?;

    Ok((
        input,
        Class {
            modifiers,
            name,
            type_params,
            extend_opt,
            implements,
            body,
            def_opt: RefCell::new(None),
            id: id_gen.get_next("class", name.fragment),
        },
    ))
}

pub fn parse_prefix<'def, 'r>(input: Tokens<'def, 'r>) -> ParseResult<'def, 'r, Span<'def>> {
    keyword("class")(input)
}

//#[cfg(test)]
//mod tests {
//    use parse::tree::{
//        Annotated, Class, ClassBody, ClassType, CompilationUnitItem, Keyword, MarkerAnnotated,
//        Modifier, TypeArg, TypeParam,
//    };
//    use parse::{compilation_unit, Tokens};
//    use std::cell::RefCell;
//    use test_common::{generate_tokens, primitive, span};
//
//    #[test]
//    fn test_bare() {
//        assert_eq!(
//            compilation_unit::parse_item(&generate_tokens(
//                r#"
//@Anno private class Test extends Super {}
//            "#
//            )),
//            Ok((
//                &[] as Tokens,
//                CompilationUnitItem::Class(Class {
//                    modifiers: vec![
//                        Modifier::Annotated(Annotated::Marker(MarkerAnnotated {
//                            class: ClassType {
//                                prefix_opt: None,
//                                name: span(1, 2, "Anno"),
//                                type_args_opt: None,
//                                def_opt: None
//                            }
//                        })),
//                        Modifier::Keyword(Keyword {
//                            name: span(1, 7, "private")
//                        })
//                    ],
//                    name: span(1, 21, "Test"),
//                    type_params: vec![],
//                    extend_opt: Some(ClassType {
//                        prefix_opt: None,
//                        name: span(1, 34, "Super"),
//                        type_args_opt: None,
//                        def_opt: None
//                    }),
//                    implements: vec![],
//                    body: ClassBody { items: vec![] },
//                    def_opt: RefCell::new(None)
//                })
//            ))
//        );
//    }
//
//    #[test]
//    fn test_type_params() {
//        assert_eq!(
//            compilation_unit::parse_item(&generate_tokens(
//                r#"
//class Test<A> implements Super, Super2<A> {}
//            "#
//            )),
//            Ok((
//                &[] as Tokens,
//                CompilationUnitItem::Class(Class {
//                    modifiers: vec![],
//                    name: span(1, 7, "Test"),
//                    type_params: vec![TypeParam {
//                        name: span(1, 12, "A"),
//                        extends: vec![]
//                    }],
//                    extend_opt: None,
//                    implements: vec![
//                        ClassType {
//                            prefix_opt: None,
//                            name: span(1, 26, "Super"),
//                            type_args_opt: None,
//                            def_opt: None
//                        },
//                        ClassType {
//                            prefix_opt: None,
//                            name: span(1, 33, "Super2"),
//                            type_args_opt: Some(vec![TypeArg::Class(ClassType {
//                                prefix_opt: None,
//                                name: span(1, 40, "A"),
//                                type_args_opt: None,
//                                def_opt: None
//                            })]),
//                            def_opt: None
//                        },
//                    ],
//                    body: ClassBody { items: vec![] },
//                    def_opt: RefCell::new(None)
//                })
//            ))
//        );
//    }
//}
