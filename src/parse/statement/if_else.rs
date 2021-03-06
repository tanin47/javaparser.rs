use parse::combinator::{keyword, symbol};
use parse::id_gen::IdGen;
use parse::statement::block;
use parse::tree::{IfElse, Statement};
use parse::{expr, ParseResult, Tokens};

pub fn parse<'def, 'r>(
    input: Tokens<'def, 'r>,
    id_gen: &mut IdGen,
) -> ParseResult<'def, 'r, Statement<'def>> {
    let (input, _) = keyword("if")(input)?;

    let (input, _) = symbol('(')(input)?;
    let (input, cond) = expr::parse(input, id_gen)?;

    let (input, _) = symbol(')')(input)?;

    let (input, block) = block::parse_block_or_single_statement(input, id_gen)?;

    let (input, else_block_opt) = match keyword("else")(input) {
        Ok((input, _)) => {
            let (input, else_block) = block::parse_block_or_single_statement(input, id_gen)?;
            (input, Some(else_block))
        }
        Err(_) => (input, None),
    };

    Ok((
        input,
        Statement::IfElse(IfElse {
            cond,
            block,
            else_block_opt,
        }),
    ))
}

//#[cfg(test)]
//mod tests {
//    use super::parse;
//    use parse::tree::{Block, Expr, IfElse, Int, LiteralString, ReturnStmt, Statement};
//    use parse::Tokens;
//    use test_common::{generate_tokens, span};
//
//    #[test]
//    fn test_if() {
//        assert_eq!(
//            parse(&generate_tokens(
//                r#"
//if (1) {
//    return;
//}
//            "#
//            )),
//            Ok((
//                &[] as Tokens,
//                Statement::IfElse(IfElse {
//                    cond: Expr::Int(Int {
//                        value: span(1, 5, "1")
//                    }),
//                    block: Block {
//                        stmts: vec![Statement::Return(ReturnStmt { expr_opt: None })]
//                    },
//                    else_block_opt: None
//                })
//            ))
//        );
//    }
//
//    #[test]
//    fn test_if_else() {
//        assert_eq!(
//            parse(&generate_tokens(
//                r#"
//if (1) {
//    return;
//} else {
//    1;
//}
//            "#
//            )),
//            Ok((
//                &[] as Tokens,
//                Statement::IfElse(IfElse {
//                    cond: Expr::Int(Int {
//                        value: span(1, 5, "1")
//                    }),
//                    block: Block {
//                        stmts: vec![Statement::Return(ReturnStmt { expr_opt: None })]
//                    },
//                    else_block_opt: Some(Block {
//                        stmts: vec![Statement::Expr(Expr::Int(Int {
//                            value: span(4, 5, "1")
//                        }))]
//                    })
//                })
//            ))
//        );
//    }
//
//    #[test]
//    fn test_if_else_if() {
//        assert_eq!(
//            parse(&generate_tokens(
//                r#"
//if (1) {
//    return;
//} else if (2) {
//    1;
//} else {
//    "a";
//}
//            "#
//            )),
//            Ok((
//                &[] as Tokens,
//                Statement::IfElse(IfElse {
//                    cond: Expr::Int(Int {
//                        value: span(1, 5, "1")
//                    }),
//                    block: Block {
//                        stmts: vec![Statement::Return(ReturnStmt { expr_opt: None })]
//                    },
//                    else_block_opt: Some(Block {
//                        stmts: vec![Statement::IfElse(IfElse {
//                            cond: Expr::Int(Int {
//                                value: span(3, 12, "2")
//                            }),
//                            block: Block {
//                                stmts: vec![Statement::Expr(Expr::Int(Int {
//                                    value: span(4, 5, "1")
//                                }))]
//                            },
//                            else_block_opt: Some(Block {
//                                stmts: vec![Statement::Expr(Expr::String(LiteralString {
//                                    value: span(6, 5, "\"a\"")
//                                }))]
//                            })
//                        })]
//                    })
//                })
//            ))
//        );
//    }
//
//    #[test]
//    fn test_dangling_if() {
//        assert_eq!(
//            parse(&generate_tokens(
//                r#"
//if (1)
//  if (2) return 1;
//  else return 2;
//            "#
//            )),
//            Ok((
//                &[] as Tokens,
//                Statement::IfElse(IfElse {
//                    cond: Expr::Int(Int {
//                        value: span(1, 5, "1")
//                    }),
//                    block: Block {
//                        stmts: vec![Statement::IfElse(IfElse {
//                            cond: Expr::Int(Int {
//                                value: span(2, 7, "2")
//                            }),
//                            block: Block {
//                                stmts: vec![Statement::Return(ReturnStmt {
//                                    expr_opt: Some(Expr::Int(Int {
//                                        value: span(2, 17, "1")
//                                    }))
//                                })]
//                            },
//                            else_block_opt: Some(Block {
//                                stmts: vec![Statement::Return(ReturnStmt {
//                                    expr_opt: Some(Expr::Int(Int {
//                                        value: span(3, 15, "2")
//                                    }))
//                                })]
//                            })
//                        })]
//                    },
//                    else_block_opt: None
//                })
//            ))
//        );
//    }
//}
