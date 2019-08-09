use nom::IResult;
use syntax::expr::atom::{method_call, name};
use syntax::expr::{precedence_7, precedence_8};
use syntax::tree::{BinaryOperation, Expr, Span};
use syntax::{comment, expr, tag};

pub fn parse_tail<'a>(left: Expr<'a>, input: Span<'a>) -> IResult<Span<'a>, Expr<'a>> {
    if let Ok((input, operator)) = tag("&&")(input) {
        let (input, right) = precedence_7::parse(input)?;

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

pub fn parse(input: Span) -> IResult<Span, Expr> {
    let (input, left) = precedence_7::parse(input)?;
    parse_tail(left, input)
}
