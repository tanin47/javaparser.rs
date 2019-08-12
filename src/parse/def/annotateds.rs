use parse::combinator::{identifier, many0, opt, separated_nonempty_list, symbol};
use parse::tree::{Annotated, AnnotatedParam, MarkerAnnotated, NormalAnnotated, SingleAnnotated};
use parse::{expr, ParseResult, Tokens};
use tokenize::span::Span;

fn name(original: Tokens) -> ParseResult<Span> {
    let (input, name) = identifier(original)?;

    if name.fragment == "interface" {
        Err(original)
    } else {
        Ok((input, name))
    }
}

fn parse_param(input: Tokens) -> ParseResult<AnnotatedParam> {
    let (input, name) = name(input)?;
    let (input, _) = symbol("=")(input)?;
    let (input, expr) = expr::parse(input)?;

    Ok((input, AnnotatedParam { name, expr }))
}

pub fn parse_annotated(input: Tokens) -> ParseResult<Annotated> {
    let (input, _) = symbol("@")(input)?;
    let (input, name) = name(input)?;

    if let Ok((input, _)) = symbol("(")(input) {
        if let Ok((input, _)) = symbol(")")(input) {
            Ok((
                input,
                Annotated::Normal(NormalAnnotated {
                    name,
                    params: vec![],
                }),
            ))
        } else if let Ok((input, params)) = separated_nonempty_list(symbol(","), parse_param)(input)
        {
            let (input, _) = symbol(")")(input)?;
            Ok((input, Annotated::Normal(NormalAnnotated { name, params })))
        } else {
            let (input, expr) = expr::parse(input)?;
            let (input, _) = symbol(")")(input)?;
            Ok((input, Annotated::Single(SingleAnnotated { name, expr })))
        }
    } else {
        Ok((input, Annotated::Marker(MarkerAnnotated { name })))
    }
}

pub fn parse(input: Tokens) -> ParseResult<Vec<Annotated>> {
    many0(parse_annotated)(input)
}

#[cfg(test)]
mod tests {
    use super::parse;
    use parse::tree::{
        Annotated, AnnotatedParam, Expr, Int, MarkerAnnotated, NormalAnnotated, SingleAnnotated,
    };
    use parse::Tokens;
    use test_common::{code, primitive, span};

    #[test]
    fn test() {
        assert_eq!(
            parse(&code(
                r#"
@Anno
@Anno()
@Anno(1)
@Anno(number=1)
            "#
            )),
            Ok((
                &[] as Tokens,
                vec![
                    Annotated::Marker(MarkerAnnotated {
                        name: span(1, 2, "Anno")
                    }),
                    Annotated::Normal(NormalAnnotated {
                        name: span(2, 2, "Anno"),
                        params: vec![]
                    }),
                    Annotated::Single(SingleAnnotated {
                        name: span(3, 2, "Anno"),
                        expr: Expr::Int(Int {
                            value: span(3, 7, "1")
                        })
                    }),
                    Annotated::Normal(NormalAnnotated {
                        name: span(4, 2, "Anno"),
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
