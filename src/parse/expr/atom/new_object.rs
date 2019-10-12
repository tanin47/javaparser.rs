use parse::combinator::{identifier, keyword, opt, separated_list, symbol};
use parse::def::class_body;
use parse::id_gen::IdGen;
use parse::tpe::type_args;
use parse::tree::{ClassType, Expr, NewObject, TypeArg};
use parse::{expr, tpe, ParseResult, Tokens};

pub fn parse_tail<'def, 'r>(
    prefix_opt: Option<Expr<'def>>,
    input: Tokens<'def, 'r>,
    id_gen: &mut IdGen,
) -> ParseResult<'def, 'r, Expr<'def>> {
    let (input, constructor_type_args_opt) = type_args::parse(input)?;

    parse_tail2(prefix_opt, input, constructor_type_args_opt, id_gen)
}

pub fn parse_tail2<'def, 'r>(
    prefix_opt: Option<Expr<'def>>,
    input: Tokens<'def, 'r>,
    constructor_type_args_opt: Option<Vec<TypeArg<'def>>>,
    id_gen: &mut IdGen,
) -> ParseResult<'def, 'r, Expr<'def>> {
    let (input, tpe) = tpe::class::parse_no_array(input)?;

    parse_tail3(prefix_opt, input, constructor_type_args_opt, tpe, id_gen)
}

pub fn parse_tail3<'def, 'r>(
    prefix_opt: Option<Expr<'def>>,
    input: Tokens<'def, 'r>,
    constructor_type_args_opt: Option<Vec<TypeArg<'def>>>,
    tpe: ClassType<'def>,
    id_gen: &mut IdGen,
) -> ParseResult<'def, 'r, Expr<'def>> {
    let (input, _) = symbol('(')(input)?;
    let (input, args) = separated_list(symbol(','), |i| expr::parse(i, id_gen))(input)?;
    let (input, _) = symbol(')')(input)?;

    let (input, body_opt) = opt(|i| class_body::parse(i, id_gen))(input)?;

    Ok((
        input,
        Expr::NewObject(NewObject {
            prefix_opt: prefix_opt.map(Box::new),
            tpe,
            constructor_type_args_opt,
            args,
            body_opt,
        }),
    ))
}

//#[cfg(test)]
//mod tests {
//    use parse::expr::atom;
//    use parse::tree::{ClassBody, ClassType, Expr, Int, LiteralString, NewObject, TypeArg};
//    use parse::Tokens;
//    use test_common::{generate_tokens, primitive, span};
//
//    #[test]
//    fn test_type_args() {
//        assert_eq!(
//            atom::parse(&generate_tokens(
//                r#"
//new <String>Test<Integer>()
//            "#
//                .trim()
//            )),
//            Ok((
//                &[] as Tokens,
//                Expr::NewObject(NewObject {
//                    prefix_opt: None,
//                    tpe: ClassType {
//                        prefix_opt: None,
//                        name: span(1, 13, "Test"),
//                        type_args_opt: Some(vec![TypeArg::Class(ClassType {
//                            prefix_opt: None,
//                            name: span(1, 18, "Integer"),
//                            type_args_opt: None,
//                            def_opt: None
//                        })]),
//                        def_opt: None
//                    },
//                    constructor_type_args_opt: Some(vec![TypeArg::Class(ClassType {
//                        prefix_opt: None,
//                        name: span(1, 6, "String"),
//                        type_args_opt: None,
//                        def_opt: None
//                    })]),
//                    args: vec![],
//                    body_opt: None
//                })
//            ))
//        );
//    }
//
//    #[test]
//    fn test_implicit_type() {
//        assert_eq!(
//            atom::parse(&generate_tokens(
//                r#"
//new Test<>()
//            "#
//            )),
//            Ok((
//                &[] as Tokens,
//                Expr::NewObject(NewObject {
//                    prefix_opt: None,
//                    tpe: ClassType {
//                        prefix_opt: None,
//                        name: span(1, 5, "Test"),
//                        type_args_opt: Some(vec![]),
//                        def_opt: None
//                    },
//                    constructor_type_args_opt: None,
//                    args: vec![],
//                    body_opt: None
//                })
//            ))
//        );
//    }
//
//    #[test]
//    fn test_bare() {
//        assert_eq!(
//            atom::parse(&generate_tokens(
//                r#"
//new Test()
//            "#
//            )),
//            Ok((
//                &[] as Tokens,
//                Expr::NewObject(NewObject {
//                    prefix_opt: None,
//                    tpe: ClassType {
//                        prefix_opt: None,
//                        name: span(1, 5, "Test"),
//                        type_args_opt: None,
//                        def_opt: None
//                    },
//                    constructor_type_args_opt: None,
//                    args: vec![],
//                    body_opt: None
//                })
//            ))
//        );
//    }
//
//    #[test]
//    fn test_with_args() {
//        assert_eq!(
//            atom::parse(&generate_tokens(
//                r#"
//new Test(1, "a")
//            "#
//            )),
//            Ok((
//                &[] as Tokens,
//                Expr::NewObject(NewObject {
//                    prefix_opt: None,
//                    tpe: ClassType {
//                        prefix_opt: None,
//                        name: span(1, 5, "Test"),
//                        type_args_opt: None,
//                        def_opt: None
//                    },
//                    constructor_type_args_opt: None,
//                    args: vec![
//                        Expr::Int(Int {
//                            value: span(1, 10, "1")
//                        }),
//                        Expr::String(LiteralString {
//                            value: span(1, 13, "\"a\"")
//                        })
//                    ],
//                    body_opt: None
//                })
//            ))
//        );
//    }
//
//    #[test]
//    fn test_anonymous() {
//        assert_eq!(
//            atom::parse(&generate_tokens(
//                r#"
//new Test() {
//}
//            "#
//            )),
//            Ok((
//                &[] as Tokens,
//                Expr::NewObject(NewObject {
//                    prefix_opt: None,
//                    tpe: ClassType {
//                        prefix_opt: None,
//                        name: span(1, 5, "Test"),
//                        type_args_opt: None,
//                        def_opt: None
//                    },
//                    constructor_type_args_opt: None,
//                    args: vec![],
//                    body_opt: Some(ClassBody { items: vec![] })
//                })
//            ))
//        );
//    }
//}
