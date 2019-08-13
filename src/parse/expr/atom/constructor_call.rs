use parse::combinator::any_keyword;
use parse::expr::atom::method_call;
use parse::tpe::type_args;
use parse::tree::{Expr, SuperConstructorCall, ThisConstructorCall};
use parse::{ParseResult, Tokens};
use tokenize::span::Span;

fn parse_this_or_super(input: Tokens) -> ParseResult<Span> {
    let (input, keyword) = any_keyword(input)?;

    match keyword.fragment {
        "this" | "super" => Ok((input, keyword)),
        _ => Err(input),
    }
}

pub fn parse(input: Tokens) -> ParseResult<Expr> {
    let (input, type_args_opt) = type_args::parse(input)?;
    let (input, this_or_super) = parse_this_or_super(input)?;
    let (input, args) = method_call::parse_args(input)?;

    match this_or_super.fragment {
        "this" => Ok((
            input,
            Expr::ThisConstructorCall(ThisConstructorCall {
                type_args_opt,
                name: this_or_super,
                args,
            }),
        )),
        "super" => Ok((
            input,
            Expr::SuperConstructorCall(SuperConstructorCall {
                type_args_opt,
                name: this_or_super,
                args,
            }),
        )),
        _ => Err(input),
    }
}
