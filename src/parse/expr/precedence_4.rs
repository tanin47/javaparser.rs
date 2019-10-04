use parse::combinator::symbol2;
use parse::expr::precedence_5;
use parse::tree::{BinaryOperation, Expr};
use parse::{ParseResult, Tokens};

pub fn parse_tail<'def, 'r>(
    left: Expr<'def>,
    input: Tokens<'def, 'r>,
) -> ParseResult<'def, 'r, Expr<'def>> {
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

pub fn parse<'def, 'r>(input: Tokens<'def, 'r>) -> ParseResult<'def, 'r, Expr<'def>> {
    let (input, left) = precedence_5::parse(input)?;
    parse_tail(left, input)
}
