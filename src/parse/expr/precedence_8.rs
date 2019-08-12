use parse::combinator::symbol2;
use parse::expr::precedence_9;
use parse::tree::{BinaryOperation, Expr};
use parse::{ParseResult, Tokens};
use tokenize::span::Span;

fn op(input: Tokens) -> ParseResult<Span> {
    if let Ok(ok) = symbol2('=', '=')(input) {
        Ok(ok)
    } else if let Ok(ok) = symbol2('!', '=')(input) {
        Ok(ok)
    } else {
        Err(input)
    }
}

pub fn parse_tail<'a>(left: Expr<'a>, input: Tokens<'a>) -> ParseResult<'a, Expr<'a>> {
    if let Ok((input, operator)) = op(input) {
        let (input, right) = precedence_9::parse(input)?;

        let expr = Expr::BinaryOperation(BinaryOperation {
            left: Box::new(left),
            operator,
            right: Box::new(right),
        });

        parse_tail(expr, input)
    } else {
        Ok((input, left))
    }
}

pub fn parse(input: Tokens) -> ParseResult<Expr> {
    let (input, left) = precedence_9::parse(input)?;
    parse_tail(left, input)
}
