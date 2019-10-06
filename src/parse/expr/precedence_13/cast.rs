use parse::combinator::{separated_nonempty_list, symbol};
use parse::expr::precedence_13;
use parse::tree::{Cast, Expr};
use parse::{tpe, ParseResult, Tokens};

pub fn parse<'def, 'r>(input: Tokens<'def, 'r>) -> ParseResult<'def, 'r, Expr<'def>> {
    let (input, _) = symbol('(')(input)?;
    let (input, tpes) = separated_nonempty_list(symbol('&'), tpe::parse)(input)?;
    let (input, _) = symbol(')')(input)?;
    let (input, expr) = precedence_13::parse(input)?;

    Ok((
        input,
        Expr::Cast(Cast {
            tpes,
            expr: Box::new(expr),
        }),
    ))
}

//#[cfg(test)]
//mod tests {
//    use test_common::{generate_tokens, primitive, span};
//
//    use super::parse;
//    use parse::tree::{Cast, ClassType, Expr, Name, Type};
//    use parse::Tokens;
//
//    #[test]
//    fn test_multi() {
//        assert_eq!(
//            parse(&generate_tokens(
//                r#"
//(boolean)(Int)t
//            "#
//            )),
//            Ok((
//                &[] as Tokens,
//                Expr::Cast(Cast {
//                    tpes: vec![primitive(1, 2, "boolean")],
//                    expr: Box::new(Expr::Cast(Cast {
//                        tpes: vec![Type::Class(ClassType {
//                            prefix_opt: None,
//                            name: span(1, 11, "Int"),
//                            type_args_opt: None,
//                            def_opt: None
//                        })],
//                        expr: Box::new(Expr::Name(Name {
//                            name: span(1, 15, "t")
//                        })),
//                    })),
//                })
//            ))
//        );
//    }
//}
