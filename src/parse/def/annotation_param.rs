use parse::combinator::{keyword, symbol};
use parse::tpe::array;
use parse::tree::{AnnotationParam, Expr, Modifier, Type};
use parse::{expr, ParseResult, Tokens};
use tokenize::span::Span;

fn parse_default(input: Tokens) -> ParseResult<Option<Expr>> {
    match keyword("default")(input) {
        Ok((input, _)) => {
            let (input, default) = expr::parse(input)?;
            Ok((input, Some(default)))
        }
        Err(_) => Ok((input, None)),
    }
}

pub fn parse<'a>(
    input: Tokens<'a>,
    modifiers: Vec<Modifier<'a>>,
    tpe: Type<'a>,
    name: Span<'a>,
) -> ParseResult<'a, AnnotationParam<'a>> {
    let (input, _) = symbol('(')(input)?;
    let (input, _) = symbol(')')(input)?;

    let (input, tpe) = array::parse_tail(input, tpe)?;
    let (input, default_opt) = parse_default(input)?;

    let (input, _) = symbol(';')(input)?;

    Ok((
        input,
        AnnotationParam {
            modifiers,
            tpe,
            name,
            default_opt,
        },
    ))
}

#[cfg(test)]
mod tests {
    use parse::def::annotation_body;
    use parse::tree::{
        Annotated, AnnotationBodyItem, AnnotationParam, ArrayType, ClassType, Expr, Int, Keyword,
        MarkerAnnotated, Modifier, Type,
    };
    use parse::Tokens;
    use test_common::{generate_tokens, primitive, span};

    #[test]
    fn test_full() {
        assert_eq!(
            annotation_body::parse_item(&generate_tokens(
                r#"
@Anno public abstract int field()[] default 1;
            "#
            )),
            Ok((
                &[] as Tokens,
                AnnotationBodyItem::Param(AnnotationParam {
                    modifiers: vec![
                        Modifier::Annotated(Annotated::Marker(MarkerAnnotated {
                            class: ClassType {
                                prefix_opt: None,
                                name: span(1, 2, "Anno"),
                                type_args_opt: None,
                                def_opt: None
                            }
                        })),
                        Modifier::Keyword(Keyword {
                            name: span(1, 7, "public")
                        }),
                        Modifier::Keyword(Keyword {
                            name: span(1, 14, "abstract")
                        })
                    ],
                    name: span(1, 27, "field"),
                    tpe: Type::Array(ArrayType {
                        tpe: Box::new(primitive(1, 23, "int")),
                        size_opt: None
                    }),
                    default_opt: Some(Expr::Int(Int {
                        value: span(1, 45, "1")
                    }))
                })
            ))
        );
    }

    #[test]
    fn test() {
        assert_eq!(
            annotation_body::parse_item(&generate_tokens(
                r#"
int field();
            "#
            )),
            Ok((
                &[] as Tokens,
                AnnotationBodyItem::Param(AnnotationParam {
                    modifiers: vec![],
                    name: span(1, 5, "field"),
                    tpe: primitive(1, 1, "int"),
                    default_opt: None
                })
            ))
        );
    }
}
