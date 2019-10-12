use parse::combinator::{identifier, keyword, opt, separated_list, symbol};
use parse::def::{class, class_body, enum_constant, modifiers};
use parse::id_gen::IdGen;
use parse::tree::{ClassBody, Enum, Modifier};
use parse::{ParseResult, Tokens};
use tokenize::span::Span;

pub fn parse_tail<'def, 'r>(
    input: Tokens<'def, 'r>,
    modifiers: Vec<Modifier<'def>>,
    id_gen: &mut IdGen,
) -> ParseResult<'def, 'r, Enum<'def>> {
    let (input, name) = identifier(input)?;

    let (input, implements) = class::parse_implements(input)?;

    let (input, _) = symbol('{')(input)?;

    let (input, constants) =
        separated_list(symbol(','), |i| enum_constant::parse(i, id_gen))(input)?;

    let (input, _) = opt(symbol(','))(input)?;

    let (input, body_opt) = match symbol(';')(input) {
        Ok((input, _)) => {
            let (input, items) = class_body::parse_items(input, id_gen)?;
            (input, Some(ClassBody { items }))
        }
        Err(_) => (input, None),
    };

    let (input, _) = symbol('}')(input)?;
    let (input, _) = opt(symbol(';'))(input)?;

    Ok((
        input,
        Enum {
            modifiers,
            name,
            implements,
            constants,
            body_opt,
        },
    ))
}

pub fn parse_prefix<'def, 'r>(input: Tokens<'def, 'r>) -> ParseResult<'def, 'r, Span<'def>> {
    keyword("enum")(input)
}

//#[cfg(test)]
//mod tests {
//    use parse::tree::{
//        Annotated, ClassBody, ClassBodyItem, ClassType, CompilationUnitItem, Enum, EnumConstant,
//        FieldDeclarators, Keyword, MarkerAnnotated, Modifier, VariableDeclarator,
//    };
//    use parse::{compilation_unit, Tokens};
//    use std::cell::RefCell;
//    use test_common::{generate_tokens, primitive, span};
//
//    #[test]
//    fn test() {
//        assert_eq!(
//            compilation_unit::parse_item(&generate_tokens(
//                r#"
//@Anno private enum Test implements Super {
//  FIRST_CONSTANT;
//  int a;
//}
//            "#
//            )),
//            Ok((
//                &[] as Tokens,
//                CompilationUnitItem::Enum(Enum {
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
//                    name: span(1, 20, "Test"),
//                    implements: vec![ClassType {
//                        prefix_opt: None,
//                        name: span(1, 36, "Super"),
//                        type_args_opt: None,
//                        def_opt: None
//                    }],
//                    constants: vec![EnumConstant {
//                        annotateds: vec![],
//                        name: span(2, 3, "FIRST_CONSTANT"),
//                        args_opt: None,
//                        body_opt: None
//                    }],
//                    body_opt: Some(ClassBody {
//                        items: vec![ClassBodyItem::FieldDeclarators(FieldDeclarators {
//                            modifiers: vec![],
//                            declarators: vec![VariableDeclarator {
//                                tpe: RefCell::new(primitive(3, 3, "int")),
//                                name: span(3, 7, "a"),
//                                expr_opt: None
//                            }]
//                        })]
//                    })
//                })
//            ))
//        );
//    }
//}
