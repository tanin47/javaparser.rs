use parse::combinator::{keyword, symbol};
use parse::id_gen::IdGen;
use parse::tree::{Statement, Throw};
use parse::{expr, ParseResult, Tokens};

pub fn parse<'def, 'r>(
    input: Tokens<'def, 'r>,
    id_gen: &mut IdGen,
) -> ParseResult<'def, 'r, Statement<'def>> {
    let (input, _) = keyword("throw")(input)?;

    let (input, expr) = expr::parse(input, id_gen)?;

    let (input, _) = symbol(';')(input)?;

    Ok((input, Statement::Throw(Throw { expr })))
}

//#[cfg(test)]
//mod tests {
//    use super::parse;
//    use parse::tree::{ClassType, Expr, NewObject, Statement, Throw};
//    use parse::Tokens;
//    use test_common::{generate_tokens, span};
//
//    #[test]
//    fn test_throw() {
//        assert_eq!(
//            parse(&generate_tokens(
//                r#"
//throw new Exception();
//            "#
//            )),
//            Ok((
//                &[] as Tokens,
//                Statement::Throw(Throw {
//                    expr: Expr::NewObject(NewObject {
//                        prefix_opt: None,
//                        tpe: ClassType {
//                            prefix_opt: None,
//                            name: span(1, 11, "Exception"),
//                            type_args_opt: None,
//                            def_opt: None
//                        },
//                        constructor_type_args_opt: None,
//                        args: vec![],
//                        body_opt: None
//                    })
//                })
//            ))
//        );
//    }
//}
