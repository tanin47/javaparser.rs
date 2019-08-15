use parse::combinator::{any_symbol, get_and_not_followed_by, symbol};
use parse::expr::precedence_6;
use parse::tree::{BinaryOperation, Expr};
use parse::{ParseResult, Tokens};

pub fn parse_tail<'a>(left: Expr<'a>, input: Tokens<'a>) -> ParseResult<'a, Expr<'a>> {
    if let Ok((input, operator)) = get_and_not_followed_by(symbol('|'), any_symbol("|="))(input) {
        let (input, right) = precedence_6::parse(input)?;

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
    let (input, left) = precedence_6::parse(input)?;
    parse_tail(left, input)
}
