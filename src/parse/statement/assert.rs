use parse::combinator::{keyword, symbol};
use parse::id_gen::IdGen;
use parse::tree::{Assert, Statement};
use parse::{expr, ParseResult, Tokens};

pub fn parse<'def, 'r>(
    input: Tokens<'def, 'r>,
    id_gen: &mut IdGen,
) -> ParseResult<'def, 'r, Statement<'def>> {
    let (input, _) = keyword("assert")(input)?;
    let (input, expr) = expr::parse(input, id_gen)?;

    let (input, error_opt) = if let Ok((input, _)) = symbol(':')(input) {
        let (input, error) = expr::parse(input, id_gen)?;
        (input, Some(error))
    } else {
        (input, None)
    };

    let (input, _) = symbol(';')(input)?;

    Ok((input, Statement::Assert(Assert { expr, error_opt })))
}

//#[cfg(test)]
//mod tests {
//    use super::parse;
//    use parse::tree::{Assert, Boolean, Expr, LiteralString, Statement};
//    use parse::Tokens;
//    use test_common::{generate_tokens, span};
//
//    #[test]
//    fn test_bare() {
//        assert_eq!(
//            parse(&generate_tokens(
//                r#"
//assert true;
//            "#
//            )),
//            Ok((
//                &[] as Tokens,
//                Statement::Assert(Assert {
//                    expr: Expr::Boolean(Boolean {
//                        value: span(1, 8, "true")
//                    }),
//                    error_opt: None
//                })
//            ))
//        );
//    }
//
//    #[test]
//    fn test_error() {
//        assert_eq!(
//            parse(&generate_tokens(
//                r#"
//assert true : "error";
//            "#
//            )),
//            Ok((
//                &[] as Tokens,
//                Statement::Assert(Assert {
//                    expr: Expr::Boolean(Boolean {
//                        value: span(1, 8, "true")
//                    }),
//                    error_opt: Some(Expr::String(LiteralString {
//                        value: span(1, 15, "\"error\"")
//                    }))
//                })
//            ))
//        );
//    }
//}
