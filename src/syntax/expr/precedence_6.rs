use nom::bytes::complete::is_not;
use nom::IResult;
use syntax::expr::precedence_7;
use syntax::tag_and_followed_by;
use syntax::tree::{BinaryOperation, Expr, Span};

pub fn parse_tail<'a>(left: Expr<'a>, input: Span<'a>) -> IResult<Span<'a>, Expr<'a>> {
    if let Ok((input, operator)) = tag_and_followed_by("^", is_not("="))(input) {
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
