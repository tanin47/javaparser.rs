use nom::branch::alt;
use nom::bytes::complete::is_not;
use nom::combinator::peek;
use nom::sequence::tuple;
use nom::IResult;
use syntax::expr::atom::{method_call, name};
use syntax::expr::precedence_13;
use syntax::tree::{BinaryOperation, Expr, Span};
use syntax::{comment, expr, tag, tag_and_followed_by};

fn op(input: Tokens) -> ParseResult<Span> {
    alt((
        tag_and_followed_by("*", is_not("=")),
        tag_and_followed_by("/", is_not("=")),
        tag_and_followed_by("%", is_not("=")),
    ))(input)
}

pub fn parse_tail<'a>(left: Expr<'a>, input: Span<'a>) -> IResult<Span<'a>, Expr<'a>> {
    if let Ok((input, operator)) = op(input) {
        let (input, right) = precedence_13::parse(input)?;

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
    let (input, left) = precedence_13::parse(input)?;
    parse_tail(left, input)
}
