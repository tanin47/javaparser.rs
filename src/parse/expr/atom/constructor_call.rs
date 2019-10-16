use parse::combinator::any_keyword;
use parse::expr::atom::method_call;
use parse::id_gen::IdGen;
use parse::tpe::type_args;
use parse::tree::{Expr, SuperConstructorCall, ThisConstructorCall};
use parse::{ParseResult, Tokens};
use tokenize::span::Span;

fn parse_this_or_super<'def, 'r>(input: Tokens<'def, 'r>) -> ParseResult<'def, 'r, Span<'def>> {
    let (input, keyword) = any_keyword(input)?;

    match keyword.fragment {
        "this" | "super" => Ok((input, keyword)),
        _ => Err(input),
    }
}

pub fn parse<'def, 'r>(
    input: Tokens<'def, 'r>,
    id_gen: &mut IdGen,
) -> ParseResult<'def, 'r, Expr<'def>> {
    let (input, type_args_opt) = type_args::parse(input)?;
    let (input, this_or_super) = parse_this_or_super(input)?;
    let (input, args) = method_call::parse_args(input, id_gen)?;

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
                prefix_opt: None,
                type_args_opt,
                name: this_or_super,
                args,
            }),
        )),
        _ => Err(input),
    }
}
