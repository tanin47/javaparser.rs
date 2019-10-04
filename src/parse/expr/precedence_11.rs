use parse::combinator::{any_symbol, get_and_not_followed_by, symbol};
use parse::expr::precedence_12;
use parse::tree::{BinaryOperation, Expr};
use parse::{ParseResult, Tokens};
use tokenize::span::Span;

fn op<'def, 'r>(input: Tokens<'def, 'r>) -> ParseResult<'def, 'r, Span<'def>> {
    if let Ok(ok) = get_and_not_followed_by(symbol('+'), any_symbol("+="))(input) {
        Ok(ok)
    } else if let Ok(ok) = get_and_not_followed_by(symbol('-'), any_symbol("-="))(input) {
        Ok(ok)
    } else {
        Err(input)
    }
}

pub fn parse_tail<'def, 'r>(
    left: Expr<'def>,
    input: Tokens<'def, 'r>,
) -> ParseResult<'def, 'r, Expr<'def>> {
    if let Ok((input, operator)) = op(input) {
        let (input, right) = precedence_12::parse(input)?;

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
    let (input, left) = precedence_12::parse(input)?;
    parse_tail(left, input)
}

#[cfg(test)]
mod tests {
    use test_common::{generate_tokens, span};

    use super::parse;
    use parse::tree::{BinaryOperation, Expr, Name, UnaryOperation};
    use parse::Tokens;

    #[test]
    fn test_increment() {
        assert_eq!(
            parse(&generate_tokens(
                r#"
a + ++b
            "#
            )),
            Ok((
                &[] as Tokens,
                Expr::BinaryOperation(BinaryOperation {
                    left: Box::new(Expr::Name(Name {
                        name: span(1, 1, "a")
                    })),
                    operator: span(1, 3, "+"),
                    right: Box::new(Expr::UnaryOperation(UnaryOperation {
                        expr: Box::new(Expr::Name(Name {
                            name: span(1, 7, "b")
                        })),
                        operator: span(1, 5, "++"),
                        is_post: false
                    }))
                })
            ))
        );
    }
}
