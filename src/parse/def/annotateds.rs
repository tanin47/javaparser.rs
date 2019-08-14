use parse::combinator::{identifier, many0, opt, separated_nonempty_list, symbol};
use parse::tpe::class;
use parse::tree::{
    Annotated, AnnotatedParam, ClassType, MarkerAnnotated, NormalAnnotated, SingleAnnotated,
};
use parse::{expr, ParseResult, Tokens};
use tokenize::span::Span;

fn parse_param(input: Tokens) -> ParseResult<AnnotatedParam> {
    let (input, name) = identifier(input)?;
    let (input, _) = symbol('=')(input)?;
    let (input, expr) = expr::parse(input)?;

    Ok((input, AnnotatedParam { name, expr }))
}

fn parse_class<'a>(
    prefix_opt: Option<ClassType<'a>>,
    input: Tokens<'a>,
) -> ParseResult<'a, ClassType<'a>> {
    let (input, name) = identifier(input)?;

    if let Ok((input, _)) = symbol('.')(input) {
        parse_class(
            Some(ClassType {
                prefix_opt: prefix_opt.map(Box::new),
                name,
                type_args_opt: None,
            }),
            input,
        )
    } else {
        Ok((
            input,
            ClassType {
                prefix_opt: prefix_opt.map(Box::new),
                name,
                type_args_opt: None,
            },
        ))
    }
}

pub fn parse_annotated(input: Tokens) -> ParseResult<Annotated> {
    let (input, _) = symbol('@')(input)?;
    let (input, class) = parse_class(None, input)?;

    if let Ok((input, _)) = symbol('(')(input) {
        if let Ok((input, _)) = symbol(')')(input) {
            Ok((
                input,
                Annotated::Normal(NormalAnnotated {
                    class,
                    params: vec![],
                }),
            ))
        } else if let Ok((input, params)) = separated_nonempty_list(symbol(','), parse_param)(input)
        {
            let (input, _) = symbol(')')(input)?;
            Ok((input, Annotated::Normal(NormalAnnotated { class, params })))
        } else {
            let (input, expr) = expr::parse(input)?;
            let (input, _) = symbol(')')(input)?;
            Ok((input, Annotated::Single(SingleAnnotated { class, expr })))
        }
    } else {
        Ok((input, Annotated::Marker(MarkerAnnotated { class })))
    }
}

pub fn parse(input: Tokens) -> ParseResult<Vec<Annotated>> {
    many0(parse_annotated)(input)
}

#[cfg(test)]
mod tests {
    use super::parse;
    use parse::tree::{
        Annotated, AnnotatedParam, ClassType, Expr, Int, MarkerAnnotated, NormalAnnotated,
        SingleAnnotated,
    };
    use parse::Tokens;
    use test_common::{code, primitive, span};

    #[test]
    fn test() {
        assert_eq!(
            parse(&code(
                r#"
@Parent.Anno
@Anno()
@Anno(1)
@Anno(number=1)
            "#
            )),
            Ok((
                &[] as Tokens,
                vec![
                    Annotated::Marker(MarkerAnnotated {
                        class: ClassType {
                            prefix_opt: Some(Box::new(ClassType {
                                prefix_opt: None,
                                name: span(1, 2, "Parent"),
                                type_args_opt: None
                            })),
                            name: span(1, 9, "Anno"),
                            type_args_opt: None
                        }
                    }),
                    Annotated::Normal(NormalAnnotated {
                        class: ClassType {
                            prefix_opt: None,
                            name: span(2, 2, "Anno"),
                            type_args_opt: None
                        },
                        params: vec![]
                    }),
                    Annotated::Single(SingleAnnotated {
                        class: ClassType {
                            prefix_opt: None,
                            name: span(3, 2, "Anno"),
                            type_args_opt: None
                        },
                        expr: Expr::Int(Int {
                            value: span(3, 7, "1")
                        })
                    }),
                    Annotated::Normal(NormalAnnotated {
                        class: ClassType {
                            prefix_opt: None,
                            name: span(4, 2, "Anno"),
                            type_args_opt: None
                        },
                        params: vec![AnnotatedParam {
                            name: span(4, 7, "number"),
                            expr: Expr::Int(Int {
                                value: span(4, 14, "1")
                            })
                        }]
                    }),
                ]
            ))
        );
    }
}
