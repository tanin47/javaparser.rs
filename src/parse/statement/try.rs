use parse::combinator::{identifier, keyword, many0, opt, separated_nonempty_list, symbol};
use parse::statement::{block, variable_declarators};
use parse::tree::{Catch, StandaloneVariableDeclarator, Statement, Try};
use parse::{tpe, ParseResult, Tokens};

fn parse_catch(input: Tokens) -> ParseResult<Catch> {
    let (input, _) = keyword("catch")(input)?;
    let (input, _) = symbol('(')(input)?;
    let (input, class_types) =
        separated_nonempty_list(symbol('|'), tpe::class::parse_no_array)(input)?;
    let (input, param_name) = identifier(input)?;
    let (input, _) = symbol(')')(input)?;

    let (input, block) = block::parse_block(input)?;

    Ok((
        input,
        Catch {
            param_name,
            class_types,
            block,
        },
    ))
}

fn parse_resources(input: Tokens) -> ParseResult<Vec<StandaloneVariableDeclarator>> {
    let (input, _) = match symbol('(')(input) {
        Ok(ok) => ok,
        Err(_) => return Ok((input, vec![])),
    };
    let (input, resources) =
        separated_nonempty_list(symbol(';'), variable_declarators::parse_standalone)(input)?;
    let (input, _) = opt(symbol(';'))(input)?;
    let (input, _) = symbol(')')(input)?;

    Ok((input, resources))
}

pub fn parse(input: Tokens) -> ParseResult<Statement> {
    let (input, _) = keyword("try")(input)?;
    let (input, resources) = parse_resources(input)?;
    let (input, try) = block::parse_block(input)?;

    let (input, catches) = many0(parse_catch)(input)?;

    let (input, finally_opt) = if let Ok((input, _)) = keyword("finally")(input) {
        let (input, finally) = block::parse_block(input)?;
        (input, Some(finally))
    } else {
        (input, None)
    };

    Ok((
        input,
        Statement::Try(Try {
            try,
            resources,
            catches,
            finally_opt,
        }),
    ))
}

#[cfg(test)]
mod tests {
    use super::parse;
    use parse::tree::{
        Block, Catch, ClassType, Expr, Int, MethodCall, Name, PrimitiveType,
        StandaloneVariableDeclarator, Statement, Throw, Try, Type, UnaryOperation,
    };
    use parse::Tokens;
    use test_common::{code, span};

    #[test]
    fn test_multiple_catches() {
        assert_eq!(
            parse(&code(
                r#"
try (
  int i = 1;
  int a = 2
) {
    i++;
} catch (Exception | Exception2 e) {
    throw e;
} catch (Exp e) {
    e.run();
} finally {
    final_method();
}
            "#
            )),
            Ok((
                &[] as Tokens,
                Statement::Try(Try {
                    try: Block {
                        stmts: vec![Statement::Expr(Expr::UnaryOperation(UnaryOperation {
                            expr: Box::new(Expr::Name(Name {
                                name: span(5, 5, "i")
                            })),
                            operator: span(5, 6, "++"),
                            is_post: true
                        }))]
                    },
                    resources: vec![
                        StandaloneVariableDeclarator {
                            modifiers: vec![],
                            tpe: Type::Primitive(PrimitiveType {
                                name: span(2, 3, "int")
                            }),
                            name: span(2, 7, "i"),
                            expr_opt: Some(Expr::Int(Int {
                                value: span(2, 11, "1")
                            }))
                        },
                        StandaloneVariableDeclarator {
                            modifiers: vec![],
                            tpe: Type::Primitive(PrimitiveType {
                                name: span(3, 3, "int")
                            }),
                            name: span(3, 7, "a"),
                            expr_opt: Some(Expr::Int(Int {
                                value: span(3, 11, "2")
                            }))
                        },
                    ],
                    catches: vec![
                        Catch {
                            param_name: span(6, 33, "e"),
                            class_types: vec![
                                ClassType {
                                    prefix_opt: None,
                                    name: span(6, 10, "Exception"),
                                    type_args_opt: None
                                },
                                ClassType {
                                    prefix_opt: None,
                                    name: span(6, 22, "Exception2"),
                                    type_args_opt: None
                                }
                            ],
                            block: Block {
                                stmts: vec![Statement::Throw(Throw {
                                    expr: Expr::Name(Name {
                                        name: span(7, 11, "e")
                                    })
                                })]
                            }
                        },
                        Catch {
                            param_name: span(8, 14, "e"),
                            class_types: vec![ClassType {
                                prefix_opt: None,
                                name: span(8, 10, "Exp"),
                                type_args_opt: None
                            }],
                            block: Block {
                                stmts: vec![Statement::Expr(Expr::MethodCall(MethodCall {
                                    prefix_opt: Some(Box::new(Expr::Name(Name {
                                        name: span(9, 5, "e")
                                    }))),
                                    name: span(9, 7, "run"),
                                    type_args_opt: None,
                                    args: vec![]
                                }))]
                            }
                        }
                    ],
                    finally_opt: Some(Block {
                        stmts: vec![Statement::Expr(Expr::MethodCall(MethodCall {
                            prefix_opt: None,
                            name: span(11, 5, "final_method"),
                            type_args_opt: None,
                            args: vec![]
                        }))]
                    })
                })
            ))
        );
    }
}
