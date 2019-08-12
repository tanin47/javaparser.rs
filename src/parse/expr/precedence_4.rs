use parse::combinator::symbol2;
use parse::expr::precedence_5;
use parse::tree::{BinaryOperation, Expr};
use parse::{ParseResult, Tokens};

pub fn parse_tail<'a>(left: Expr<'a>, input: Tokens<'a>) -> ParseResult<'a, Expr<'a>> {
    if let Ok((input, operator)) = symbol2('&', '&')(input) {
        let (input, right) = precedence_5::parse(input)?;

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
    let (input, left) = precedence_5::parse(input)?;
    parse_tail(left, input)
}
