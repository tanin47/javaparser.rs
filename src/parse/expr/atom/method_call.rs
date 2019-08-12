use parse::combinator::{separated_list, symbol};
use parse::tpe::type_args;
use parse::tree::{Expr, MethodCall, TypeArg};
use parse::{expr, ParseResult, Tokens};
use tokenize::span::Span;

pub fn parse_args(input: Tokens) -> ParseResult<Vec<Expr>> {
    let (input, _) = symbol('(')(input)?;
    let (input, args) = separated_list(symbol(','), expr::parse)(input)?;
    let (input, _) = symbol(')')(input)?;

    Ok((input, args))
}

pub fn parse_tail<'a>(
    input: Tokens<'a>,
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

pub fn parse(is_next: bool) -> impl Fn(Tokens) -> ParseResult<MethodCall> {
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

#[cfg(test)]
mod tests {
    use super::parse;
    use parse::Tokens;
    use syntax::tree::{
        Expr, Int, Lambda, LiteralString, Method, MethodCall, Name, Param, ReturnStmt, Type,
    };
    use test_common::{code, span};

    #[test]
    fn test_bare() {
        assert_eq!(
            parse(false)(&code(
                r#"
method()
            "#
            )),
            Ok((
                &[] as Tokens,
                MethodCall {
                    name: span(1, 1, "method"),
                    type_args_opt: None,
                    args: vec![],
                }
            ))
        );
    }

    #[test]
    fn test_with_args() {
        assert_eq!(
            parse(false)(&code(
                r#"
method(1, "a")
            "#
            )),
            Ok((
                &[] as Tokens,
                MethodCall {
                    name: span(1, 1, "method"),
                    type_args_opt: None,
                    args: vec![
                        Expr::Int(Int {
                            value: span(1, 8, "1")
                        }),
                        Expr::String(LiteralString {
                            value: span(1, 12, "a")
                        }),
                    ],
                }
            ))
        );
    }

    #[test]
    fn test_lambda() {
        assert_eq!(
            parse(false)(&code(
                r#"
method(1, (x) -> 2)
            "#
            )),
            Ok((
                &[] as Tokens,
                MethodCall {
                    name: span(1, 1, "method"),
                    type_args_opt: None,
                    args: vec![
                        Expr::Int(Int {
                            value: span(1, 8, "1")
                        }),
                        Expr::Lambda(Lambda {
                            params: vec![Param {
                                annotateds: vec![],
                                tpe: Type::UnknownType,
                                name: span(1, 12, "x")
                            }],
                            expr_opt: Some(Box::new(Expr::Int(Int {
                                value: span(1, 18, "2")
                            }))),
                            block_opt: None
                        }),
                    ],
                }
            ))
        );
    }
}
