use parse::combinator::symbol2;
use parse::expr::precedence_14;
use parse::id_gen::IdGen;
use parse::tree::{Expr, UnaryOperation};
use parse::{ParseResult, Tokens};
use tokenize::span::Span;

fn op<'def, 'r>(input: Tokens<'def, 'r>) -> ParseResult<'def, 'r, Span<'def>> {
    if let Ok(ok) = symbol2('+', '+')(input) {
        Ok(ok)
    } else if let Ok(ok) = symbol2('-', '-')(input) {
        Ok(ok)
    } else {
        Err(input)
    }
}

pub fn parse<'def, 'r>(
    input: Tokens<'def, 'r>,
    id_gen: &mut IdGen,
) -> ParseResult<'def, 'r, Expr<'def>> {
    let (input, operator) = op(input)?;
    let (input, expr) = precedence_14::parse(input, id_gen)?;

    Ok((
        input,
        Expr::UnaryOperation(UnaryOperation {
            expr: Box::new(expr),
            operator,
            is_post: false,
        }),
    ))
}

//#[cfg(test)]
//mod tests {
//    use test_common::{generate_tokens, span};
//
//    use super::parse;
//    use parse::tree::{Expr, Name, UnaryOperation};
//    use parse::Tokens;
//
//    #[test]
//    fn test_increment() {
//        assert_eq!(
//            parse(&generate_tokens(
//                r#"
//++abc
//            "#
//            )),
//            Ok((
//                &[] as Tokens,
//                Expr::UnaryOperation(UnaryOperation {
//                    expr: Box::new(Expr::Name(Name {
//                        name: span(1, 3, "abc")
//                    })),
//                    operator: span(1, 1, "++"),
//                    is_post: false
//                })
//            ))
//        );
//    }
//
//    #[test]
//    fn test_decrement() {
//        assert_eq!(
//            parse(&generate_tokens(
//                r#"
//--abc
//            "#
//            )),
//            Ok((
//                &[] as Tokens,
//                Expr::UnaryOperation(UnaryOperation {
//                    expr: Box::new(Expr::Name(Name {
//                        name: span(1, 3, "abc")
//                    })),
//                    operator: span(1, 1, "--"),
//                    is_post: false
//                })
//            ))
//        );
//    }
//}
