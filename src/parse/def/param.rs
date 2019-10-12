use parse::combinator::{identifier, opt, symbol};
use parse::def::modifiers;
use parse::id_gen::IdGen;
use parse::tpe::array;
use parse::tree::Param;
use parse::{tpe, ParseResult, Tokens};
use tokenize::span::Span;

pub fn parse_varargs<'def, 'r>(input: Tokens<'def, 'r>) -> ParseResult<'def, 'r, ()> {
    let (input, _) = symbol('.')(input)?;
    let (input, _) = symbol('.')(input)?;
    let (input, _) = symbol('.')(input)?;
    Ok((input, ()))
}

pub fn parse<'def, 'r>(
    input: Tokens<'def, 'r>,
    id_gen: &mut IdGen,
) -> ParseResult<'def, 'r, Param<'def>> {
    let (input, modifiers) = modifiers::parse(input, id_gen)?;
    let (input, tpe) = tpe::parse(input)?;
    let (input, varargs_opt) = opt(parse_varargs)(input)?;
    let (input, name) = identifier(input)?;
    let (input, tpe) = array::parse_tail(input, tpe)?;

    Ok((
        input,
        Param {
            modifiers,
            tpe,
            is_varargs: varargs_opt.is_some(),
            name,
        },
    ))
}

//#[cfg(test)]
//mod tests {
//    use super::parse;
//    use parse::tree::{
//        Annotated, ArrayType, ClassType, Keyword, MarkerAnnotated, Modifier, Param, Type,
//    };
//    use parse::Tokens;
//    use test_common::{generate_tokens, span};
//
//    #[test]
//    fn test_class() {
//        assert_eq!(
//            parse(&generate_tokens(
//                r#"
//final @Anno Test... t
//            "#
//            )),
//            Ok((
//                &[] as Tokens,
//                Param {
//                    modifiers: vec![
//                        Modifier::Keyword(Keyword {
//                            name: span(1, 1, "final")
//                        }),
//                        Modifier::Annotated(Annotated::Marker(MarkerAnnotated {
//                            class: ClassType {
//                                prefix_opt: None,
//                                name: span(1, 8, "Anno"),
//                                type_args_opt: None,
//                                def_opt: None
//                            }
//                        })),
//                    ],
//                    tpe: Type::Class(ClassType {
//                        prefix_opt: None,
//                        name: span(1, 13, "Test"),
//                        type_args_opt: None,
//                        def_opt: None
//                    }),
//                    is_varargs: true,
//                    name: span(1, 21, "t"),
//                }
//            ))
//        );
//    }
//
//    #[test]
//    fn test_array() {
//        assert_eq!(
//            parse(&generate_tokens(
//                r#"
//Test[] t[]
//            "#
//            )),
//            Ok((
//                &[] as Tokens,
//                Param {
//                    modifiers: vec![],
//                    tpe: Type::Array(ArrayType {
//                        tpe: Box::new(Type::Array(ArrayType {
//                            tpe: Box::new(Type::Class(ClassType {
//                                prefix_opt: None,
//                                name: span(1, 1, "Test"),
//                                type_args_opt: None,
//                                def_opt: None
//                            })),
//                            size_opt: None
//                        })),
//                        size_opt: None
//                    }),
//                    is_varargs: false,
//                    name: span(1, 8, "t"),
//                }
//            ))
//        );
//    }
//}
