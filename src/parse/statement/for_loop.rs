use parse::combinator::{keyword, opt, separated_list, symbol};
use parse::id_gen::IdGen;
use parse::statement::block::parse_block_or_single_statement;
use parse::statement::variable_declarators;
use parse::tree::{ForLoop, Foreach, Statement};
use parse::{expr, statement, ParseResult, Tokens};

fn parse_foreach<'def, 'r>(
    input: Tokens<'def, 'r>,
    id_gen: &mut IdGen,
) -> ParseResult<'def, 'r, Statement<'def>> {
    let (input, declarator) = variable_declarators::parse_standalone(input, id_gen)?;

    let (input, _) = symbol(':')(input)?;
    let (input, expr) = expr::parse(input, id_gen)?;
    let (input, _) = symbol(')')(input)?;
    let (input, block) = parse_block_or_single_statement(input, id_gen)?;

    Ok((
        input,
        Statement::Foreach(Foreach {
            declarator,
            expr,
            block,
        }),
    ))
}

fn parse_inits<'def, 'r>(
    input: Tokens<'def, 'r>,
    id_gen: &mut IdGen,
) -> ParseResult<'def, 'r, Vec<Statement<'def>>> {
    if let Ok((input, declarators)) = variable_declarators::parse_without_semicolon(input, id_gen) {
        Ok((input, vec![declarators]))
    } else {
        separated_list(symbol(','), |i| {
            statement::expr::parse_without_semicolon(i, id_gen)
        })(input)
    }
}

fn parse_for_loop<'def, 'r>(
    input: Tokens<'def, 'r>,
    id_gen: &mut IdGen,
) -> ParseResult<'def, 'r, Statement<'def>> {
    let (input, inits) = parse_inits(input, id_gen)?;

    let (input, _) = symbol(';')(input)?;
    let (input, cond_opt) = opt(|i| expr::parse(i, id_gen))(input)?;
    let (input, _) = symbol(';')(input)?;
    let (input, updates) = separated_list(symbol(','), |i| {
        statement::expr::parse_without_semicolon(i, id_gen)
    })(input)?;

    let (input, _) = symbol(')')(input)?;
    let (input, block) = parse_block_or_single_statement(input, id_gen)?;

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

pub fn parse<'def, 'r>(
    original: Tokens<'def, 'r>,
    id_gen: &mut IdGen,
) -> ParseResult<'def, 'r, Statement<'def>> {
    let (input, _) = keyword("for")(original)?;
    let (input, _) = symbol('(')(input)?;

    if let Ok(ok) = parse_foreach(input, id_gen) {
        Ok(ok)
    } else if let Ok(ok) = parse_for_loop(input, id_gen) {
        Ok(ok)
    } else {
        Err(original)
    }
}

//#[cfg(test)]
//mod tests {
//    use super::parse;
//    use parse::tree::{
//        Assigned, Assignment, BinaryOperation, Block, Expr, ForLoop, Foreach, Int, Name,
//        PrimitiveType, PrimitiveTypeType, ReturnStmt, StandaloneVariableDeclarator, Statement,
//        Type, UnaryOperation, VariableDeclarator, VariableDeclarators,
//    };
//    use parse::Tokens;
//    use std::cell::RefCell;
//    use test_common::{generate_tokens, span};
//
//    #[test]
//    fn test_foreach() {
//        assert_eq!(
//            parse(&generate_tokens(
//                r#"
//for(int a:list) a++;
//            "#
//            )),
//            Ok((
//                &[] as Tokens,
//                Statement::Foreach(Foreach {
//                    declarator: StandaloneVariableDeclarator {
//                        modifiers: vec![],
//                        tpe: RefCell::new(Type::Primitive(PrimitiveType {
//                            name: span(1, 5, "int"),
//                            tpe: PrimitiveTypeType::Int
//                        })),
//                        name: span(1, 9, "a"),
//                        expr_opt: None
//                    },
//                    expr: Expr::Name(Name {
//                        name: span(1, 11, "list")
//                    }),
//                    block: Block {
//                        stmts: vec![Statement::Expr(Expr::UnaryOperation(UnaryOperation {
//                            expr: Box::new(Expr::Name(Name {
//                                name: span(1, 17, "a"),
//                            })),
//                            operator: span(1, 18, "++"),
//                            is_post: true
//                        }))]
//                    }
//                })
//            ))
//        );
//    }
//
//    #[test]
//    fn test_short() {
//        assert_eq!(
//            parse(&generate_tokens(
//                r#"
//for(int i=0;i<2;i++) x++;
//            "#
//            )),
//            Ok((
//                &[] as Tokens,
//                Statement::ForLoop(ForLoop {
//                    inits: vec![Statement::VariableDeclarators(VariableDeclarators {
//                        modifiers: vec![],
//                        declarators: vec![VariableDeclarator {
//                            tpe: RefCell::new(Type::Primitive(PrimitiveType {
//                                name: span(1, 5, "int"),
//                                tpe: PrimitiveTypeType::Int
//                            })),
//                            name: span(1, 9, "i"),
//                            expr_opt: Some(Expr::Int(Int {
//                                value: span(1, 11, "0")
//                            }))
//                        }]
//                    })],
//                    cond_opt: Some(Expr::BinaryOperation(BinaryOperation {
//                        left: Box::new(Expr::Name(Name {
//                            name: span(1, 13, "i")
//                        })),
//                        operator: span(1, 14, "<"),
//                        right: Box::new(Expr::Int(Int {
//                            value: span(1, 15, "2")
//                        }))
//                    })),
//                    updates: vec![Statement::Expr(Expr::UnaryOperation(UnaryOperation {
//                        expr: Box::new(Expr::Name(Name {
//                            name: span(1, 17, "i")
//                        })),
//                        operator: span(1, 18, "++"),
//                        is_post: true
//                    }))],
//                    block: Block {
//                        stmts: vec![Statement::Expr(Expr::UnaryOperation(UnaryOperation {
//                            expr: Box::new(Expr::Name(Name {
//                                name: span(1, 22, "x"),
//                            })),
//                            operator: span(1, 23, "++"),
//                            is_post: true
//                        }))]
//                    }
//                })
//            ))
//        );
//    }
//
//    #[test]
//    fn test_long() {
//        assert_eq!(
//            parse(&generate_tokens(
//                r#"
//for(;;) {
//  x = 1;
//  return;
//}
//            "#
//            )),
//            Ok((
//                &[] as Tokens,
//                Statement::ForLoop(ForLoop {
//                    inits: vec![],
//                    cond_opt: None,
//                    updates: vec![],
//                    block: Block {
//                        stmts: vec![
//                            Statement::Expr(Expr::Assignment(Assignment {
//                                assigned: Box::new(Assigned::Name(Name {
//                                    name: span(2, 3, "x"),
//                                })),
//                                operator: span(2, 5, "="),
//                                expr: Box::new(Expr::Int(Int {
//                                    value: span(2, 7, "1")
//                                }))
//                            })),
//                            Statement::Return(ReturnStmt { expr_opt: None }),
//                        ]
//                    }
//                })
//            ))
//        );
//    }
//}
