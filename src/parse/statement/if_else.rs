use parse::combinator::{symbol, word};
use parse::statement::block;
use parse::tree::{IfElse, Statement};
use parse::{expr, ParseResult, Tokens};

pub fn parse(input: Tokens) -> ParseResult<Statement> {
    let (input, _) = word("if")(input)?;

    let (input, _) = symbol('(')(input)?;
    let (input, cond) = expr::parse(input)?;

    let (input, _) = symbol(')')(input)?;

    let (input, block) = block::parse_block_or_single_statement(input)?;

    let (input, else_block_opt) = match word("else")(input) {
        Ok((input, _)) => {
            let (input, else_block) = block::parse_block_or_single_statement(input)?;
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

#[cfg(test)]
mod tests {
    use super::parse;
    use parse::tree::{Block, Expr, IfElse, Int, LiteralString, ReturnStmt, Statement};
    use parse::Tokens;
    use test_common::{code, span};

    #[test]
    fn test_if() {
        assert_eq!(
            parse(&code(
                r#"
if (1) {
    return;
}
            "#
            )),
            Ok((
                &[] as Tokens,
                Statement::IfElse(IfElse {
                    cond: Expr::Int(Int {
                        value: span(1, 5, "1")
                    }),
                    block: Block {
                        stmts: vec![Statement::Return(ReturnStmt { expr_opt: None })]
                    },
                    else_block_opt: None
                })
            ))
        );
    }

    #[test]
    fn test_if_else() {
        assert_eq!(
            parse(&code(
                r#"
if (1) {
    return;
} else {
    1;
}
            "#
            )),
            Ok((
                &[] as Tokens,
                Statement::IfElse(IfElse {
                    cond: Expr::Int(Int {
                        value: span(1, 5, "1")
                    }),
                    block: Block {
                        stmts: vec![Statement::Return(ReturnStmt { expr_opt: None })]
                    },
                    else_block_opt: Some(Block {
                        stmts: vec![Statement::Expr(Expr::Int(Int {
                            value: span(4, 5, "1")
                        }))]
                    })
                })
            ))
        );
    }

    #[test]
    fn test_if_else_if() {
        assert_eq!(
            parse(&code(
                r#"
if (1) {
    return;
} else if (2) {
    1;
} else {
    "a";
}
            "#
            )),
            Ok((
                &[] as Tokens,
                Statement::IfElse(IfElse {
                    cond: Expr::Int(Int {
                        value: span(1, 5, "1")
                    }),
                    block: Block {
                        stmts: vec![Statement::Return(ReturnStmt { expr_opt: None })]
                    },
                    else_block_opt: Some(Block {
                        stmts: vec![Statement::IfElse(IfElse {
                            cond: Expr::Int(Int {
                                value: span(3, 12, "2")
                            }),
                            block: Block {
                                stmts: vec![Statement::Expr(Expr::Int(Int {
                                    value: span(4, 5, "1")
                                }))]
                            },
                            else_block_opt: Some(Block {
                                stmts: vec![Statement::Expr(Expr::String(LiteralString {
                                    value: span(6, 5, "\"a\"")
                                }))]
                            })
                        })]
                    })
                })
            ))
        );
    }

    #[test]
    fn test_dangling_if() {
        assert_eq!(
            parse(&code(
                r#"
if (1) 
  if (2) return 1;
  else return 2;
            "#
            )),
            Ok((
                &[] as Tokens,
                Statement::IfElse(IfElse {
                    cond: Expr::Int(Int {
                        value: span(1, 5, "1")
                    }),
                    block: Block {
                        stmts: vec![Statement::IfElse(IfElse {
                            cond: Expr::Int(Int {
                                value: span(2, 7, "2")
                            }),
                            block: Block {
                                stmts: vec![Statement::Return(ReturnStmt {
                                    expr_opt: Some(Expr::Int(Int {
                                        value: span(2, 17, "1")
                                    }))
                                })]
                            },
                            else_block_opt: Some(Block {
                                stmts: vec![Statement::Return(ReturnStmt {
                                    expr_opt: Some(Expr::Int(Int {
                                        value: span(3, 15, "2")
                                    }))
                                })]
                            })
                        })]
                    },
                    else_block_opt: None
                })
            ))
        );
    }
}
