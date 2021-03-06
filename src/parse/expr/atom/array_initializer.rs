use parse::combinator::{opt, separated_list, symbol};
use parse::id_gen::IdGen;
use parse::tree::{ArrayInitializer, Expr};
use parse::{expr, ParseResult, Tokens};

pub fn parse_initializer<'def, 'r>(
    input: Tokens<'def, 'r>,
    id_gen: &mut IdGen,
) -> ParseResult<'def, 'r, ArrayInitializer<'def>> {
    let (input, _) = symbol('{')(input)?;

    let (input, items) = separated_list(symbol(','), |i| expr::parse(i, id_gen))(input)?;
    let (input, _) = opt(symbol(','))(input)?;

    let (input, _) = symbol('}')(input)?;

    Ok((input, ArrayInitializer { items }))
}

pub fn parse<'def, 'r>(
    input: Tokens<'def, 'r>,
    id_gen: &mut IdGen,
) -> ParseResult<'def, 'r, Expr<'def>> {
    let (input, init) = parse_initializer(input, id_gen)?;
    Ok((input, Expr::ArrayInitializer(init)))
}

//#[cfg(test)]
//mod tests {
//    use super::parse;
//    use parse::tree::{ArrayInitializer, Expr, Int};
//    use parse::Tokens;
//    use test_common::{generate_tokens, primitive, span};
//
//    #[test]
//    fn test() {
//        assert_eq!(
//            parse(&generate_tokens("{}")),
//            Ok((
//                &[] as Tokens,
//                Expr::ArrayInitializer(ArrayInitializer { items: vec![] })
//            ))
//        );
//    }
//
//    #[test]
//    fn test_nested() {
//        assert_eq!(
//            parse(&generate_tokens(
//                r#"
//{ 1, {2}}
//            "#
//            )),
//            Ok((
//                &[] as Tokens,
//                Expr::ArrayInitializer(ArrayInitializer {
//                    items: vec![
//                        Expr::Int(Int {
//                            value: span(1, 3, "1")
//                        }),
//                        Expr::ArrayInitializer(ArrayInitializer {
//                            items: vec![Expr::Int(Int {
//                                value: span(1, 7, "2")
//                            }),]
//                        })
//                    ]
//                })
//            ))
//        );
//    }
//}
