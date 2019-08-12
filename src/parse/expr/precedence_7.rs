use nom::bytes::complete::is_not;
use nom::IResult;
use syntax::expr::atom::{method_call, name};
use syntax::expr::precedence_8;
use syntax::tree::{BinaryOperation, Expr, Span};
use syntax::{comment, expr, tag, tag_and_followed_by};

pub fn parse_tail<'a>(left: Expr<'a>, input: Span<'a>) -> IResult<Span<'a>, Expr<'a>> {
    if let Ok((input, operator)) = tag_and_followed_by("&", is_not("&="))(input) {
        let (input, right) = precedence_8::parse(input)?;

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
    let (input, left) = precedence_8::parse(input)?;
    parse_tail(left, input)
}
