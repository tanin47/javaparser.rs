use parse::combinator::{identifier, many0, opt, separated_list, separated_nonempty_list, symbol};
use parse::id_gen::IdGen;
use parse::tpe::class;
use parse::tree::{
    Annotated, AnnotatedParam, AnnotatedValue, AnnotatedValueArray, ClassType, EnclosingType,
    MarkerAnnotated, NormalAnnotated, SingleAnnotated,
};
use parse::{expr, ParseResult, Tokens};
use std::cell::Cell;
use tokenize::span::Span;

fn parse_array_value<'def, 'r>(
    input: Tokens<'def, 'r>,
    id_gen: &mut IdGen,
) -> ParseResult<'def, 'r, AnnotatedValueArray<'def>> {
    let (input, _) = symbol('{')(input)?;
    let (input, items) = separated_list(symbol(','), |i| parse_value(i, id_gen))(input)?;
    let (input, _) = opt(symbol(','))(input)?;
    let (input, _) = symbol('}')(input)?;

    Ok((input, AnnotatedValueArray { items }))
}

fn parse_value<'def, 'r>(
    input: Tokens<'def, 'r>,
    id_gen: &mut IdGen,
) -> ParseResult<'def, 'r, AnnotatedValue<'def>> {
    if let Ok((input, annotated)) = parse_annotated(input, id_gen) {
        Ok((input, AnnotatedValue::Annotated(Box::new(annotated))))
    } else if let Ok((input, array)) = parse_array_value(input, id_gen) {
        Ok((input, AnnotatedValue::Array(array)))
    } else {
        let (input, expr) = expr::parse(input, id_gen)?;
        Ok((input, AnnotatedValue::Expr(expr)))
    }
}

fn parse_param<'def, 'r>(
    input: Tokens<'def, 'r>,
    id_gen: &mut IdGen,
) -> ParseResult<'def, 'r, AnnotatedParam<'def>> {
    let (input, name) = identifier(input)?;
    let (input, _) = symbol('=')(input)?;
    let (input, value) = parse_value(input, id_gen)?;

    Ok((input, AnnotatedParam { name, value }))
}

fn parse_class<'def, 'r>(
    prefix_opt: Option<ClassType<'def>>,
    input: Tokens<'def, 'r>,
) -> ParseResult<'def, 'r, ClassType<'def>> {
    let (input, name) = identifier(input)?;

    if let Ok((input, _)) = symbol('.')(input) {
        parse_class(
            Some(ClassType {
                prefix_opt: prefix_opt.map(|c| Box::new(EnclosingType::Class(c))),
                name: name.fragment.to_owned(),
                span_opt: Some(name),
                type_args_opt: None,
                def_opt: None,
            }),
            input,
        )
    } else {
        Ok((
            input,
            ClassType {
                prefix_opt: prefix_opt.map(|c| Box::new(EnclosingType::Class(c))),
                name: name.fragment.to_owned(),
                span_opt: Some(name),
                type_args_opt: None,
                def_opt: None,
            },
        ))
    }
}

pub fn parse_annotated<'def, 'r>(
    input: Tokens<'def, 'r>,
    id_gen: &mut IdGen,
) -> ParseResult<'def, 'r, Annotated<'def>> {
    let (input, _) = symbol('@')(input)?;
    let (input, class) = parse_class(None, input)?;

    if let Ok((input, _)) = symbol('(')(input) {
        if let Ok((input, _)) = symbol(')')(input) {
            return Ok((
                input,
                Annotated::Normal(NormalAnnotated {
                    class,
                    params: vec![],
                }),
            ));
        } else if let Ok((input, params)) =
            separated_nonempty_list(symbol(','), |i| parse_param(i, id_gen))(input)
        {
            let (input, _) = symbol(')')(input)?;
            return Ok((input, Annotated::Normal(NormalAnnotated { class, params })));
        }

        let (input, value) = parse_value(input, id_gen)?;
        let (input, _) = symbol(')')(input)?;
        return Ok((input, Annotated::Single(SingleAnnotated { class, value })));
    } else {
        return Ok((input, Annotated::Marker(MarkerAnnotated { class })));
    }
}

pub fn parse<'def, 'r>(
    input: Tokens<'def, 'r>,
    id_gen: &mut IdGen,
) -> ParseResult<'def, 'r, Vec<Annotated<'def>>> {
    many0(|i| parse_annotated(i, id_gen))(input)
}

//#[cfg(test)]
//mod tests {
//    use super::parse;
//    use parse::tree::{
//        Annotated, AnnotatedParam, AnnotatedValue, ClassType, EnclosingType, Expr, Int,
//        MarkerAnnotated, NormalAnnotated, SingleAnnotated,
//    };
//    use parse::Tokens;
//    use test_common::{generate_tokens, primitive, span};
//
//    #[test]
//    fn test() {
//        assert_eq!(
//            parse(&generate_tokens(
//                r#"
//@Parent.Anno
//@Anno()
//@Anno(1)
//@Anno(number=1)
//            "#
//            )),
//            Ok((
//                &[] as Tokens,
//                vec![
//                    Annotated::Marker(MarkerAnnotated {
//                        class: ClassType {
//                            prefix_opt: Some(Box::new(EnclosingType::Class(ClassType {
//                                prefix_opt: None,
//                                name: span(1, 2, "Parent"),
//                                type_args_opt: None,
//                                def_opt: None
//                            }))),
//                            name: span(1, 9, "Anno"),
//                            type_args_opt: None,
//                            def_opt: None
//                        }
//                    }),
//                    Annotated::Normal(NormalAnnotated {
//                        class: ClassType {
//                            prefix_opt: None,
//                            name: span(2, 2, "Anno"),
//                            type_args_opt: None,
//                            def_opt: None
//                        },
//                        params: vec![]
//                    }),
//                    Annotated::Single(SingleAnnotated {
//                        class: ClassType {
//                            prefix_opt: None,
//                            name: span(3, 2, "Anno"),
//                            type_args_opt: None,
//                            def_opt: None
//                        },
//                        value: AnnotatedValue::Expr(Expr::Int(Int {
//                            value: span(3, 7, "1")
//                        }))
//                    }),
//                    Annotated::Normal(NormalAnnotated {
//                        class: ClassType {
//                            prefix_opt: None,
//                            name: span(4, 2, "Anno"),
//                            type_args_opt: None,
//                            def_opt: None
//                        },
//                        params: vec![AnnotatedParam {
//                            name: span(4, 7, "number"),
//                            value: AnnotatedValue::Expr(Expr::Int(Int {
//                                value: span(4, 14, "1")
//                            }))
//                        }]
//                    }),
//                ]
//            ))
//        );
//    }
//}
