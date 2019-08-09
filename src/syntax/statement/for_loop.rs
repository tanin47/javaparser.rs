use nom::IResult;

use nom::branch::alt;
use nom::combinator::opt;
use nom::error::ErrorKind;
use nom::multi::separated_list;
use syntax::statement::block::parse_block_or_single_statement;
use syntax::statement::variable_declarators;
use syntax::tree::{Class, ForLoop, Foreach, Method, Statement};
use syntax::tree::{ReturnStmt, Span};
use syntax::{expr, statement, tag};

fn parse_foreach(input: Span) -> IResult<Span, Statement> {
    let (input, declarator) = variable_declarators::parse_standalone(input)?;

    let (input, _) = tag(":")(input)?;
    let (input, expr) = expr::parse(input)?;
    let (input, _) = tag(")")(input)?;
    let (input, block) = parse_block_or_single_statement(input)?;

    Ok((
        input,
        Statement::Foreach(Foreach {
            declarator,
            expr,
            block,
        }),
    ))
}

fn parse_for_loop<'a>(input: Span<'a>) -> IResult<Span<'a>, Statement<'a>> {
    let (input, inits) = alt((
        |input: Span<'a>| {
            let (input, declarators) = variable_declarators::parse_without_semicolon(input)?;
            Ok((input, vec![declarators]))
        },
        separated_list(tag(","), statement::expr::parse_without_semicolon),
    ))(input)?;

    let (input, _) = tag(";")(input)?;
    let (input, cond_opt) = opt(expr::parse)(input)?;
    let (input, _) = tag(";")(input)?;
    let (input, updates) =
        separated_list(tag(","), statement::expr::parse_without_semicolon)(input)?;

    let (input, _) = tag(")")(input)?;
    let (input, block) = parse_block_or_single_statement(input)?;

    Ok((
        input,
        Statement::ForLoop(ForLoop {
            inits,
            cond_opt,
            updates,
            block,
        }),
    ))
}

pub fn parse(input: Span) -> IResult<Span, Statement> {
    let (input, _) = tag("for")(input)?;
    let (input, _) = tag("(")(input)?;

    alt((parse_foreach, parse_for_loop))(input)
}

#[cfg(test)]
mod tests {
    use super::parse;
    use syntax::tree::{
        Assigned, Assignment, BinaryOperation, Block, Expr, ForLoop, Foreach, Int, LiteralString,
        Method, Name, PrimitiveType, ReturnStmt, StandaloneVariableDeclarator, Statement, Type,
        UnaryOperation, VariableDeclarator, VariableDeclarators,
    };
    use test_common::{code, span};

    #[test]
    fn test_foreach() {
        assert_eq!(
            parse(code(
                r#"
for(int a:list) a++;
            "#
                .trim()
            )),
            Ok((
                span(1, 21, ""),
                Statement::Foreach(Foreach {
                    declarator: StandaloneVariableDeclarator {
                        annotateds: vec![],
                        tpe: Type::Primitive(PrimitiveType {
                            name: span(1, 5, "int")
                        }),
                        name: span(1, 9, "a"),
                        expr_opt: None
                    },
                    expr: Expr::Name(Name {
                        name: span(1, 11, "list")
                    }),
                    block: Block {
                        stmts: vec![Statement::Expr(Expr::UnaryOperation(UnaryOperation {
                            expr: Box::new(Expr::Name(Name {
                                name: span(1, 17, "a"),
                            })),
                            operator: span(1, 18, "++"),
                            is_post: true
                        }))]
                    }
                })
            ))
        );
    }

    #[test]
    fn test_short() {
        assert_eq!(
            parse(code(
                r#"
for(int i=0;i<2;i++) x++;
            "#
                .trim()
            )),
            Ok((
                span(1, 26, ""),
                Statement::ForLoop(ForLoop {
                    inits: vec![Statement::VariableDeclarators(VariableDeclarators {
                        annotateds: vec![],
                        declarators: vec![VariableDeclarator {
                            tpe: Type::Primitive(PrimitiveType {
                                name: span(1, 5, "int")
                            }),
                            name: span(1, 9, "i"),
                            expr_opt: Some(Expr::Int(Int {
                                value: span(1, 11, "0")
                            }))
                        }]
                    })],
                    cond_opt: Some(Expr::BinaryOperation(BinaryOperation {
                        left: Box::new(Expr::Name(Name {
                            name: span(1, 13, "i")
                        })),
                        operator: span(1, 14, "<"),
                        right: Box::new(Expr::Int(Int {
                            value: span(1, 15, "2")
                        }))
                    })),
                    updates: vec![Statement::Expr(Expr::UnaryOperation(UnaryOperation {
                        expr: Box::new(Expr::Name(Name {
                            name: span(1, 17, "i")
                        })),
                        operator: span(1, 18, "++"),
                        is_post: true
                    }))],
                    block: Block {
                        stmts: vec![Statement::Expr(Expr::UnaryOperation(UnaryOperation {
                            expr: Box::new(Expr::Name(Name {
                                name: span(1, 22, "x"),
                            })),
                            operator: span(1, 23, "++"),
                            is_post: true
                        }))]
                    }
                })
            ))
        );
    }

    #[test]
    fn test_long() {
        assert_eq!(
            parse(code(
                r#"
for(;;) {
  x = 1;
  return;
}
            "#
                .trim()
            )),
            Ok((
                span(4, 2, ""),
                Statement::ForLoop(ForLoop {
                    inits: vec![],
                    cond_opt: None,
                    updates: vec![],
                    block: Block {
                        stmts: vec![
                            Statement::Expr(Expr::Assignment(Assignment {
                                assigned: Box::new(Assigned::Name(Name {
                                    name: span(2, 3, "x"),
                                })),
                                operator: span(2, 5, "="),
                                expr: Box::new(Expr::Int(Int {
                                    value: span(2, 7, "1")
                                }))
                            })),
                            Statement::Return(ReturnStmt { expr_opt: None }),
                        ]
                    }
                })
            ))
        );
    }
}
