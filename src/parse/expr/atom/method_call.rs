use parse::combinator::{identifier, separated_list, symbol};
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
) -> ParseResult<'a, MethodCall<'a>> {
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
    move |input: Tokens| {
        let (input, type_args_opt) = if is_next {
            type_args::parse(input)?
        } else {
            (input, None)
        };

        let (input, name) = identifier(input)?;

        parse_tail(input, None, name, type_args_opt)
    }
}

#[cfg(test)]
mod tests {
    use super::parse;
    use parse::tree::{Expr, Int, Lambda, LiteralString, MethodCall, Param, Type};
    use parse::Tokens;
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
                    prefix_opt: None,
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
                    prefix_opt: None,
                    name: span(1, 1, "method"),
                    type_args_opt: None,
                    args: vec![
                        Expr::Int(Int {
                            value: span(1, 8, "1")
                        }),
                        Expr::String(LiteralString {
                            value: span(1, 11, "\"a\"")
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
                    prefix_opt: None,
                    name: span(1, 1, "method"),
                    type_args_opt: None,
                    args: vec![
                        Expr::Int(Int {
                            value: span(1, 8, "1")
                        }),
                        Expr::Lambda(Lambda {
                            params: vec![Param {
                                modifiers: vec![],
                                tpe: Type::UnknownType,
                                is_varargs: false,
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
