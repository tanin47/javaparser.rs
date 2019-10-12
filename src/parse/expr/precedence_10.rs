use parse::combinator::{any_symbol, get_and_not_followed_by, symbol, symbol2, symbol3};
use parse::expr::precedence_11;
use parse::id_gen::IdGen;
use parse::tree::{BinaryOperation, Expr};
use parse::{ParseResult, Tokens};
use tokenize::span::Span;

fn op<'def, 'r>(input: Tokens<'def, 'r>) -> ParseResult<'def, 'r, Span<'def>> {
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

pub fn parse_tail<'def, 'r>(
    left: Expr<'def>,
    input: Tokens<'def, 'r>,
    id_gen: &mut IdGen,
) -> ParseResult<'def, 'r, Expr<'def>> {
    if let Ok((input, operator)) = op(input) {
        let (input, right) = precedence_11::parse(input, id_gen)?;

        let expr = Expr::BinaryOperation(BinaryOperation {
            left: Box::new(left),
            operator,
            right: Box::new(right),
        });

        parse_tail(expr, input, id_gen)
    } else {
        Ok((input, left))
    }
}

pub fn parse<'def, 'r>(
    input: Tokens<'def, 'r>,
    id_gen: &mut IdGen,
) -> ParseResult<'def, 'r, Expr<'def>> {
    let (input, left) = precedence_11::parse(input, id_gen)?;
    parse_tail(left, input, id_gen)
}

//#[cfg(test)]
//mod tests {
//    use test_common::{generate_tokens, span};
//
//    use super::parse;
//    use parse::tree::{BinaryOperation, Expr, Int, Name};
//    use parse::Tokens;
//
//    #[test]
//    fn test_less_than_less_than() {
//        assert_eq!(
//            parse(&generate_tokens(
//                r#"
//a << 1
//            "#
//            )),
//            Ok((
//                &[] as Tokens,
//                Expr::BinaryOperation(BinaryOperation {
//                    left: Box::new(Expr::Name(Name {
//                        name: span(1, 1, "a")
//                    })),
//                    operator: span(1, 3, "<<"),
//                    right: Box::new(Expr::Int(Int {
//                        value: span(1, 6, "1")
//                    })),
//                })
//            ))
//        );
//    }
//}
