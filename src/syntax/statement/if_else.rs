use nom::bytes::complete::{tag, take, take_till, take_while};
use nom::character::complete::multispace0;
use nom::character::is_space;
use nom::IResult;

use nom::multi::many0;
use nom::sequence::Tuple;
use syntax::statement::block;
use syntax::tree::{Block, Class, IfElse, Method, Statement};
use syntax::tree::{ReturnStmt, Span};
use syntax::{comment, expr, statement};

pub fn parse(input: Span) -> IResult<Span, Statement> {
    let (input, _) = comment::parse(input)?;
    let (input, _) = tag("if")(input)?;

    let (input, _) = comment::parse(input)?;
    let (input, _) = tag("(")(input)?;
    let (input, cond) = expr::parse(input)?;

    let (input, _) = comment::parse(input)?;
    let (input, _) = tag(")")(input)?;

    let (input, block) = block::parse_block_or_single_statement(input)?;

    let (input, _) = comment::parse(input)?;
    let (input, else_block_opt) = match tag("else")(input) as IResult<Span, Span> {
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
    use syntax::tree::{Block, Expr, IfElse, Int, LiteralString, Method, ReturnStmt, Statement};
    use test_common::{code, span};

    #[test]
    fn test_if() {
        assert_eq!(
            parse(code(
                r#"
if (1) {
    return;
}
            "#
                .trim()
            )),
            Ok((
                span(3, 2, ""),
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
            parse(code(
                r#"
if (1) {
    return;
} else {
    1;
}
            "#
                .trim()
            )),
            Ok((
                span(5, 2, ""),
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
            parse(code(
                r#"
if (1) {
    return;
} else if (2) {
    1;
} else {
    "a";
}
            "#
                .trim()
            )),
            Ok((
                span(7, 2, ""),
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
                                    value: span(6, 6, "a")
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
            parse(code(
                r#"
if (1) 
  if (2) return 1;
  else return 2;
            "#
                .trim()
            )),
            Ok((
                span(3, 17, ""),
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
