use parse::combinator::{identifier, symbol, word};
use parse::def::{annotation_body, modifiers};
use parse::tree::{Annotation, Modifier};
use parse::{ParseResult, Tokens};
use tokenize::span::Span;

pub fn parse_tail<'a>(
    input: Tokens<'a>,
    modifiers: Vec<Modifier<'a>>,
) -> ParseResult<'a, Annotation<'a>> {
    let (input, name) = identifier(input)?;

    let (input, body) = annotation_body::parse(input)?;

    Ok((
        input,
        Annotation {
            modifiers,
            name,
            body,
        },
    ))
}

pub fn parse_prefix(input: Tokens) -> ParseResult<Span> {
    let (input, _) = symbol('@')(input)?;
    word("interface")(input)
}

#[cfg(test)]
mod tests {
    use parse::tree::{
        Annotated, Annotation, AnnotationBody, CompilationUnitItem, Keyword, MarkerAnnotated,
        Modifier,
    };
    use parse::{compilation_unit, Tokens};
    use test_common::{code, primitive, span};

    #[test]
    fn test() {
        assert_eq!(
            compilation_unit::parse_item(&code(
                r#"
@Anno private @interface Test {}
            "#
            )),
            Ok((
                &[] as Tokens,
                CompilationUnitItem::Annotation(Annotation {
                    modifiers: vec![
                        Modifier::Annotated(Annotated::Marker(MarkerAnnotated {
                            name: span(1, 2, "Anno")
                        })),
                        Modifier::Keyword(Keyword {
                            name: span(1, 7, "private")
                        })
                    ],
                    name: span(1, 26, "Test"),
                    body: AnnotationBody { items: vec![] }
                })
            ))
        );
    }
}
