use parse::tree::{Expr, Int};
use parse::{ParseResult, Tokens};
use tokenize::span::Span;

pub fn parse(input: Tokens) -> ParseResult<Expr> {
    Ok((
        &input[1..],
        Expr::Int(Int {
            value: Span {
                line: 1,
                col: 1,
                fragment: "",
            },
        }),
    ))
}
