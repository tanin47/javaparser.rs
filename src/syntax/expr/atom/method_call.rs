use nom::bytes::complete::{tag, take_while};
use nom::character::complete::{alpha1, alphanumeric0};
use nom::character::{is_alphanumeric, is_digit};
use nom::combinator::map;
use nom::multi::separated_list;
use nom::sequence::tuple;
use nom::{Compare, IResult, InputLength, InputTake, InputTakeAtPosition};
use syntax::expr::atom::name;
use syntax::tpe::type_args;
use syntax::tree::{Expr, Int, LiteralString, MethodCall, Span, TypeArg};
use syntax::{comment, expr};

pub fn parse_args(input: Span) -> IResult<Span, Vec<Expr>> {
    let (input, _) = comment::parse(input)?;
    let (input, _) = tag("(")(input)?;

    let (input, args) = separated_list(tag(","), expr::parse)(input)?;

    let (input, _) = comment::parse(input)?;
    let (input, _) = tag(")")(input)?;

    Ok((input, args))
}

pub fn parse_tail<'a>(
    input: Span<'a>,
    prefix_opt: Option<Box<Expr<'a>>>,
    name: Span<'a>,
    type_args_opt: Option<Vec<TypeArg<'a>>>,
) -> IResult<Span<'a>, MethodCall<'a>> {
    let (input, args) = parse_args(input)?;

    Ok((
        input,
        MethodCall {
            prefix_opt,
            name,
            type_args_opt,
            args,
        },
    ))
}

pub fn parse(is_next: bool) -> impl Fn(Span) -> IResult<Span, MethodCall> {
    move |input: Span| {
        let (input, type_args_opt) = if is_next {
            let (input, _) = comment::parse(input)?;
            type_args::parse(input)?
        } else {
            (input, None)
        };

        let (input, _) = comment::parse(input)?;
        let (input, name) = name::identifier(input)?;

        parse_tail(input, None, name, type_args_opt)
    }
}

//#[cfg(test)]
//mod tests {
//    use super::parse;
//    use syntax::tree::{
//        Expr, Int, Lambda, LiteralString, Method, MethodCall, Name, Param,
//        ReturnStmt, Type,
//    };
//    use test_common::{code, span};
//
//    #[test]
//    fn test_bare() {
//        assert_eq!(
//            parse(false)(code(
//                r#"
//method()
//            "#
//                .trim()
//            )),
//            Ok((
//                span(1, 9, ""),
//                MethodCall {
//                    name: span(1, 1, "method"),
//                    type_args_opt: None,
//                    args: vec![],
//                }
//            ))
//        );
//    }
//
//    #[test]
//    fn test_with_args() {
//        assert_eq!(
//            parse(false)(code(
//                r#"
//method(1, "a")
//            "#
//                .trim()
//            )),
//            Ok((
//                span(1, 15, ""),
//                MethodCall {
//                    name: span(1, 1, "method"),
//                    type_args_opt: None,
//                    args: vec![
//                        Expr::Int(Int {
//                            value: span(1, 8, "1")
//                        }),
//                        Expr::String(LiteralString {
//                            value: span(1, 12, "a")
//                        }),
//                    ],
//                }
//            ))
//        );
//    }
//
//    #[test]
//    fn test_lambda() {
//        assert_eq!(
//            parse(false)(code(
//                r#"
//method(1, (x) -> 2)
//            "#
//                .trim()
//            )),
//            Ok((
//                span(1, 20, ""),
//                MethodCall {
//                    name: span(1, 1, "method"),
//                    type_args_opt: None,
//                    args: vec![
//                        Expr::Int(Int {
//                            value: span(1, 8, "1")
//                        }),
//                        Expr::Lambda(Lambda {
//                            params: vec![Param {
//                                annotateds: vec![],
//                                tpe: Type::UnknownType,
//                                name: span(1, 12, "x")
//                            }],
//                            expr_opt: Some(Box::new(Expr::Int(Int {
//                                value: span(1, 18, "2")
//                            }))),
//                            block_opt: None
//                        }),
//                    ],
//                }
//            ))
//        );
//    }
//}
