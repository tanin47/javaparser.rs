use parse::combinator::{any_symbol, get_and_not_followed_by, symbol, symbol2, symbol3};
use parse::expr::precedence_11;
use parse::tree::{BinaryOperation, Expr};
use parse::{ParseResult, Tokens};
use tokenize::span::Span;

fn op(input: Tokens) -> ParseResult<Span> {
    if let Ok(ok) = get_and_not_followed_by(symbol3('>', '>', '>'), symbol('='))(input) {
        Ok(ok)
    } else if let Ok(ok) = get_and_not_followed_by(symbol2('<', '<'), symbol('='))(input) {
        Ok(ok)
    } else if let Ok(ok) = get_and_not_followed_by(symbol2('>', '>'), any_symbol(">="))(input) {
        Ok(ok)
    } else {
        Err(input)
    }
}

pub fn parse_tail<'a>(left: Expr<'a>, input: Tokens<'a>) -> ParseResult<'a, Expr<'a>> {
    if let Ok((input, operator)) = op(input) {
        let (input, right) = precedence_11::parse(input)?;

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
    let (input, left) = precedence_11::parse(input)?;
    parse_tail(left, input)
}

#[cfg(test)]
mod tests {
    use test_common::{code, span};

    use super::parse;
    use parse::tree::{BinaryOperation, Expr, Int, Name};
    use parse::Tokens;

    #[test]
    fn test_less_than_less_than() {
        assert_eq!(
            parse(&code(
                r#"
a << 1
            "#
            )),
            Ok((
                &[] as Tokens,
                Expr::BinaryOperation(BinaryOperation {
                    left: Box::new(Expr::Name(Name {
                        name: span(1, 1, "a")
                    })),
                    operator: span(1, 3, "<<"),
                    right: Box::new(Expr::Int(Int {
                        value: span(1, 6, "1")
                    })),
                })
            ))
        );
    }
}
