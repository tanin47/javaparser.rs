use parse::combinator::{identifier, keyword, opt, separated_nonempty_list, symbol};
use parse::def::{class_body, type_params};
use parse::id_gen::IdGen;
use parse::tpe::class;
use parse::tree::{ClassType, Interface, Modifier};
use parse::{ParseResult, Tokens};
use tokenize::span::Span;

fn parse_extends<'def, 'r>(input: Tokens<'def, 'r>) -> ParseResult<'def, 'r, Vec<ClassType<'def>>> {
    if let Ok((input, _)) = keyword("extends")(input) {
        let (input, classes) = separated_nonempty_list(symbol(','), class::parse_no_array)(input)?;
        Ok((input, classes))
    } else {
        Ok((input, vec![]))
    }
}

pub fn parse_tail<'def, 'r>(
    input: Tokens<'def, 'r>,
    modifiers: Vec<Modifier<'def>>,
    id_gen: &mut IdGen,
) -> ParseResult<'def, 'r, Interface<'def>> {
    let (input, name) = identifier(input)?;
    let (input, type_params) = type_params::parse(input, id_gen)?;

    let (input, extends) = parse_extends(input)?;

    let (input, body) = class_body::parse(input, id_gen)?;
    let (input, _) = opt(symbol(';'))(input)?;

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

pub fn parse_prefix<'def, 'r>(input: Tokens<'def, 'r>) -> ParseResult<'def, 'r, Span<'def>> {
    keyword("interface")(input)
}

//#[cfg(test)]
//mod tests {
//    use parse::tree::{
//        Annotated, ClassBody, ClassType, CompilationUnitItem, Interface, Keyword, MarkerAnnotated,
//        Modifier, TypeArg, TypeParam,
//    };
//    use parse::{compilation_unit, Tokens};
//    use test_common::{generate_tokens, primitive, span};
//
//    #[test]
//    fn test_bare() {
//        assert_eq!(
//            compilation_unit::parse_item(&generate_tokens(
//                r#"
//@Anno private interface Test {}
//            "#
//            )),
//            Ok((
//                &[] as Tokens,
//                CompilationUnitItem::Interface(Interface {
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
//                    name: span(1, 25, "Test"),
//                    type_params: vec![],
//                    extends: vec![],
//                    body: ClassBody { items: vec![] }
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
//interface Test<A> extends Super, Super2<A> {}
//            "#
//            )),
//            Ok((
//                &[] as Tokens,
//                CompilationUnitItem::Interface(Interface {
//                    modifiers: vec![],
//                    name: span(1, 11, "Test"),
//                    type_params: vec![TypeParam {
//                        name: span(1, 16, "A"),
//                        extends: vec![]
//                    }],
//                    extends: vec![
//                        ClassType {
//                            prefix_opt: None,
//                            name: span(1, 27, "Super"),
//                            type_args_opt: None,
//                            def_opt: None
//                        },
//                        ClassType {
//                            prefix_opt: None,
//                            name: span(1, 34, "Super2"),
//                            type_args_opt: Some(vec![TypeArg::Class(ClassType {
//                                prefix_opt: None,
//                                name: span(1, 41, "A"),
//                                type_args_opt: None,
//                                def_opt: None
//                            })]),
//                            def_opt: None
//                        },
//                    ],
//                    body: ClassBody { items: vec![] }
//                })
//            ))
//        );
//    }
//}
