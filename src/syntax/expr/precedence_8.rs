use nom::branch::alt;
use nom::IResult;
use syntax::expr::precedence_9;
use syntax::tag;
use syntax::tree::{BinaryOperation, Expr, Span};

fn op(input: Span) -> IResult<Span, Span> {
    alt((tag("=="), tag("!=")))(input)
}

pub fn parse_tail<'a>(left: Expr<'a>, input: Span<'a>) -> IResult<Span<'a>, Expr<'a>> {
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

pub fn parse(input: Span) -> IResult<Span, Expr> {
    let (input, left) = precedence_9::parse(input)?;
    parse_tail(left, input)
}
