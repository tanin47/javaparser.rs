use parse::combinator::{identifier, opt, separated_list, symbol, word};
use parse::def::{class, class_body, enum_constant, modifiers};
use parse::tree::{ClassBody, Enum, Modifier};
use parse::{ParseResult, Tokens};
use tokenize::span::Span;

pub fn parse_tail<'a>(
    input: Tokens<'a>,
    modifiers: Vec<Modifier<'a>>,
) -> ParseResult<'a, Enum<'a>> {
    let (input, name) = identifier(input)?;

    let (input, implements) = class::parse_implements(input)?;

    let (input, _) = symbol('{')(input)?;

    let (input, constants) = separated_list(symbol(','), enum_constant::parse)(input)?;

    let (input, _) = opt(symbol(','))(input)?;

    let (input, body_opt) = match symbol(';')(input) {
        Ok((input, _)) => {
            let (input, items) = class_body::parse_items(input)?;
            (input, Some(ClassBody { items }))
        }
        Err(_) => (input, None),
    };

    let (input, _) = symbol('}')(input)?;

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

pub fn parse_prefix(input: Tokens) -> ParseResult<Span> {
    word("enum")(input)
}

#[cfg(test)]
mod tests {
    use parse::tree::{
        Annotated, ClassBody, ClassBodyItem, ClassType, CompilationUnitItem, Enum, EnumConstant,
        FieldDeclarators, Keyword, MarkerAnnotated, Modifier, VariableDeclarator,
    };
    use parse::{compilation_unit, Tokens};
    use test_common::{code, primitive, span};

    #[test]
    fn test() {
        assert_eq!(
            compilation_unit::parse_item(&code(
                r#"
@Anno private enum Test implements Super {
  FIRST_CONSTANT;
  int a;
}
            "#
            )),
            Ok((
                &[] as Tokens,
                CompilationUnitItem::Enum(Enum {
                    modifiers: vec![
                        Modifier::Annotated(Annotated::Marker(MarkerAnnotated {
                            name: span(1, 2, "Anno")
                        })),
                        Modifier::Keyword(Keyword {
                            name: span(1, 7, "private")
                        })
                    ],
                    name: span(1, 20, "Test"),
                    implements: vec![ClassType {
                        prefix_opt: None,
                        name: span(1, 36, "Super"),
                        type_args_opt: None
                    }],
                    constants: vec![EnumConstant {
                        annotateds: vec![],
                        name: span(2, 3, "FIRST_CONSTANT"),
                        args_opt: None,
                        body_opt: None
                    }],
                    body_opt: Some(ClassBody {
                        items: vec![ClassBodyItem::FieldDeclarators(FieldDeclarators {
                            modifiers: vec![],
                            declarators: vec![VariableDeclarator {
                                tpe: primitive(3, 3, "int"),
                                name: span(3, 7, "a"),
                                expr_opt: None
                            }]
                        })]
                    })
                })
            ))
        );
    }
}
