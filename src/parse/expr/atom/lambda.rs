use either::Either;
use parse::combinator::{identifier, separated_list, symbol, symbol2};
use parse::def::param;
use parse::expr::atom::name;
use parse::id_gen::IdGen;
use parse::statement::block::parse_block;
use parse::tree::{Block, Expr, Lambda, Param, Type};
use parse::{expr, ParseResult, Tokens};

fn parse_block_or_single_expr<'def, 'r>(
    input: Tokens<'def, 'r>,
    id_gen: &mut IdGen,
) -> ParseResult<'def, 'r, Either<Block<'def>, Expr<'def>>> {
    match parse_block(input, id_gen) {
        Ok((input, block)) => Ok((input, Either::Left(block))),
        Err(_) => {
            let (input, expr) = expr::parse(input, id_gen)?;
            Ok((input, Either::Right(expr)))
        }
    }
}

fn parse_param_with_type_or_without_type<'def, 'r>(
    input: Tokens<'def, 'r>,
    id_gen: &mut IdGen,
) -> ParseResult<'def, 'r, Param<'def>> {
    match param::parse(input, id_gen) {
        Ok(result) => Ok(result),
        Err(_) => {
            let (input, name) = identifier(input)?;
            Ok((
                input,
                Param {
                    modifiers: vec![],
                    tpe: Type::UnknownType,
                    is_varargs: false,
                    name,
                },
            ))
        }
    }
}

pub fn parse<'def, 'r>(
    input: Tokens<'def, 'r>,
    id_gen: &mut IdGen,
) -> ParseResult<'def, 'r, Expr<'def>> {
    let (input, params) = if let Ok((input, _)) = symbol('(')(input) {
        let (input, params) = separated_list(symbol(','), |i| {
            parse_param_with_type_or_without_type(i, id_gen)
        })(input)?;
        let (input, _) = symbol(')')(input)?;

        (input, params)
    } else if let Ok((input, Either::Right(name))) = name::parse(input) {
        (
            input,
            vec![Param {
                modifiers: vec![],
                tpe: Type::UnknownType,
                is_varargs: false,
                name: name.name,
            }],
        )
    } else {
        return Err(input);
    };

    let (input, _) = symbol2('-', '>')(input)?;

    let (input, block_or_expr) = parse_block_or_single_expr(input, id_gen)?;

    let (block_opt, expr_opt) = match block_or_expr {
        Either::Left(block) => (Some(block), None),
        Either::Right(expr) => (None, Some(Box::new(expr))),
    };

    Ok((
        input,
        Expr::Lambda(Lambda {
            inferred_method_opt: None,
            inferred_params: vec![],
            inferred_return_type: Type::UnknownType,
            params,
            expr_opt,
            block_opt,
        }),
    ))
}

//#[cfg(test)]
//mod tests {
//    use super::parse;
//    use parse::tree::{
//        Block, ClassType, Expr, Int, Lambda, Param, PrimitiveType, PrimitiveTypeType, ReturnStmt,
//        Statement, Type,
//    };
//    use parse::Tokens;
//    use test_common::{generate_tokens, span};
//
//    #[test]
//    fn test_single_with_args() {
//        assert_eq!(
//            parse(&generate_tokens(
//                r#"
//(Test t, a, int i) -> 1
//            "#
//            )),
//            Ok((
//                &[] as Tokens,
//                Expr::Lambda(Lambda {
//                    params: vec![
//                        Param {
//                            modifiers: vec![],
//                            tpe: Type::Class(ClassType {
//                                prefix_opt: None,
//                                name: span(1, 2, "Test"),
//                                type_args_opt: None,
//                                def_opt: None
//                            }),
//                            is_varargs: false,
//                            name: span(1, 7, "t"),
//                        },
//                        Param {
//                            modifiers: vec![],
//                            tpe: Type::UnknownType,
//                            is_varargs: false,
//                            name: span(1, 10, "a"),
//                        },
//                        Param {
//                            modifiers: vec![],
//                            tpe: Type::Primitive(PrimitiveType {
//                                name: span(1, 13, "int"),
//                                tpe: PrimitiveTypeType::Int
//                            }),
//                            is_varargs: false,
//                            name: span(1, 17, "i"),
//                        }
//                    ],
//                    expr_opt: Some(Box::new(Expr::Int(Int {
//                        value: span(1, 23, "1")
//                    }))),
//                    block_opt: None
//                })
//            ))
//        );
//    }
//
//    #[test]
//    fn test_simple() {
//        assert_eq!(
//            parse(&generate_tokens(
//                r#"
//(x) -> 2
//            "#
//            )),
//            Ok((
//                &[] as Tokens,
//                Expr::Lambda(Lambda {
//                    params: vec![Param {
//                        modifiers: vec![],
//                        tpe: Type::UnknownType,
//                        is_varargs: false,
//                        name: span(1, 2, "x"),
//                    }],
//                    expr_opt: Some(Box::new(Expr::Int(Int {
//                        value: span(1, 8, "2")
//                    }))),
//                    block_opt: None
//                })
//            ))
//        );
//    }
//
//    #[test]
//    fn test_simple2() {
//        assert_eq!(
//            parse(&generate_tokens(
//                r#"
//x -> 2
//            "#
//            )),
//            Ok((
//                &[] as Tokens,
//                Expr::Lambda(Lambda {
//                    params: vec![Param {
//                        modifiers: vec![],
//                        tpe: Type::UnknownType,
//                        is_varargs: false,
//                        name: span(1, 1, "x"),
//                    }],
//                    expr_opt: Some(Box::new(Expr::Int(Int {
//                        value: span(1, 6, "2")
//                    }))),
//                    block_opt: None
//                })
//            ))
//        );
//    }
//
//    #[test]
//    fn test_block() {
//        assert_eq!(
//            parse(&generate_tokens(
//                r#"
//() -> { return 1; }
//            "#
//            )),
//            Ok((
//                &[] as Tokens,
//                Expr::Lambda(Lambda {
//                    params: vec![],
//                    expr_opt: None,
//                    block_opt: Some(Block {
//                        stmts: vec![Statement::Return(ReturnStmt {
//                            expr_opt: Some(Expr::Int(Int {
//                                value: span(1, 16, "1")
//                            }))
//                        })]
//                    })
//                })
//            ))
//        );
//    }
//}
