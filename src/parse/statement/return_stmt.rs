use parse::combinator::{keyword, symbol};
use parse::id_gen::IdGen;
use parse::tree::{ReturnStmt, Statement};
use parse::{expr, ParseResult, Tokens};

pub fn parse<'def, 'r>(
    input: Tokens<'def, 'r>,
    id_gen: &mut IdGen,
) -> ParseResult<'def, 'r, Statement<'def>> {
    let (input, _) = keyword("return")(input)?;

    let (input, expr_opt) = match symbol(';')(input) {
        Ok((input, _)) => (input, None),
        Err(_) => {
            let (input, expr) = expr::parse(input, id_gen)?;
            let (input, _) = symbol(';')(input)?;
            (input, Some(expr))
        }
    };

    Ok((input, Statement::Return(ReturnStmt { expr_opt })))
}

//#[cfg(test)]
//mod tests {
//    use super::parse;
//    use parse::tree::{Expr, LiteralString, ReturnStmt, Statement};
//    use parse::Tokens;
//    use test_common::{generate_tokens, span};
//
//    #[test]
//    fn test_return_void() {
//        assert_eq!(
//            parse(&generate_tokens(
//                r#"
//return;
//            "#
//            )),
//            Ok((
//                &[] as Tokens,
//                Statement::Return(ReturnStmt { expr_opt: None })
//            ))
//        );
//    }
//
//    #[test]
//    fn test_return_string() {
//        assert_eq!(
//            parse(&generate_tokens(
//                r#"
//return "test";
//            "#
//            )),
//            Ok((
//                &[] as Tokens,
//                Statement::Return(ReturnStmt {
//                    expr_opt: Some(Expr::String(LiteralString {
//                        value: span(1, 8, "\"test\"")
//                    }))
//                })
//            ))
//        );
//    }
//}
