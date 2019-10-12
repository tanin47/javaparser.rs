use parse::combinator::{identifier, keyword, symbol};
use parse::def::{annotation_body, modifiers};
use parse::id_gen::IdGen;
use parse::tree::{Annotation, Modifier};
use parse::{ParseResult, Tokens};
use tokenize::span::Span;

pub fn parse_tail<'def, 'r>(
    input: Tokens<'def, 'r>,
    modifiers: Vec<Modifier<'def>>,
    id_gen: &mut IdGen,
) -> ParseResult<'def, 'r, Annotation<'def>> {
    let (input, name) = identifier(input)?;

    let (input, body) = annotation_body::parse(input, id_gen)?;

    Ok((
        input,
        Annotation {
            modifiers,
            name,
            body,
        },
    ))
}

pub fn parse_prefix<'def, 'r>(input: Tokens<'def, 'r>) -> ParseResult<'def, 'r, Span<'def>> {
    let (input, _) = symbol('@')(input)?;
    keyword("interface")(input)
}

//#[cfg(test)]
//mod tests {
//    use parse::tree::{
//        Annotated, Annotation, AnnotationBody, ClassType, CompilationUnitItem, Keyword,
//        MarkerAnnotated, Modifier,
//    };
//    use parse::{compilation_unit, Tokens};
//    use test_common::{generate_tokens, primitive, span};
//
//    #[test]
//    fn test() {
//        assert_eq!(
//            compilation_unit::parse_item(&generate_tokens(
//                r#"
//@Anno private @interface Test {}
//            "#
//            )),
//            Ok((
//                &[] as Tokens,
//                CompilationUnitItem::Annotation(Annotation {
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
//                    name: span(1, 26, "Test"),
//                    body: AnnotationBody { items: vec![] }
//                })
//            ))
//        );
//    }
//}
