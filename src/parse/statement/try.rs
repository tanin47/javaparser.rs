use parse::combinator::{identifier, keyword, many0, opt, separated_nonempty_list, symbol};
use parse::def::modifiers;
use parse::statement::{block, variable_declarators};
use parse::tree::{Catch, Expr, StandaloneVariableDeclarator, Statement, Try, TryResource};
use parse::{expr, tpe, ParseResult, Tokens};

fn parse_catch<'def, 'r>(input: Tokens<'def, 'r>) -> ParseResult<'def, 'r, Catch<'def>> {
    let (input, _) = keyword("catch")(input)?;
    let (input, _) = symbol('(')(input)?;
    let (input, modifiers) = modifiers::parse(input)?;
    let (input, class_types) =
        separated_nonempty_list(symbol('|'), tpe::class::parse_no_array)(input)?;
    let (input, param_name) = identifier(input)?;
    let (input, _) = symbol(')')(input)?;

    let (input, block) = block::parse_block(input)?;

    Ok((
        input,
        Catch {
            modifiers,
            param_name,
            class_types,
            block,
        },
    ))
}

// TODO: This can be optimized to do bottom-up parsing.
fn parse_resource<'def, 'r>(input: Tokens<'def, 'r>) -> ParseResult<'def, 'r, TryResource<'def>> {
    if let Ok((input, declarator)) = variable_declarators::parse_standalone(input) {
        return Ok((input, TryResource::Declarator(declarator)));
    } else if let Ok((input, expr)) = expr::parse(input) {
        match expr {
            Expr::Name(name) => return Ok((input, TryResource::Name(name))),
            Expr::FieldAccess(field) => return Ok((input, TryResource::FieldAccess(field))),
            _ => (),
        }
    }

    Err(input)
}

fn parse_resources<'def, 'r>(
    input: Tokens<'def, 'r>,
) -> ParseResult<'def, 'r, Vec<TryResource<'def>>> {
    let (input, _) = match symbol('(')(input) {
        Ok(ok) => ok,
        Err(_) => return Ok((input, vec![])),
    };
    let (input, resources) = separated_nonempty_list(symbol(';'), parse_resource)(input)?;
    let (input, _) = opt(symbol(';'))(input)?;
    let (input, _) = symbol(')')(input)?;

    Ok((input, resources))
}

pub fn parse<'def, 'r>(input: Tokens<'def, 'r>) -> ParseResult<'def, 'r, Statement<'def>> {
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
        Block, Catch, ClassType, Expr, FieldAccess, Int, Keyword, MethodCall, Modifier, Name,
        PrimitiveType, PrimitiveTypeType, StandaloneVariableDeclarator, Statement, Throw, Try,
        TryResource, Type, UnaryOperation,
    };
    use parse::Tokens;
    use std::cell::RefCell;
    use test_common::{generate_tokens, span};

    #[test]
    fn test_only_try() {
        assert_eq!(
            parse(&generate_tokens(
                r#"
try (in; a.b) {
}
            "#
            )),
            Ok((
                &[] as Tokens,
                Statement::Try(Try {
                    try: Block { stmts: vec![] },
                    resources: vec![
                        TryResource::Name(Name {
                            name: span(1, 6, "in")
                        }),
                        TryResource::FieldAccess(FieldAccess {
                            expr: Box::new(Expr::Name(Name {
                                name: span(1, 10, "a")
                            })),
                            field: Name {
                                name: span(1, 12, "b")
                            },
                            tpe_opt: RefCell::new(None)
                        }),
                    ],
                    catches: vec![],
                    finally_opt: None
                })
            ))
        );
    }

    #[test]
    fn test_multiple_catches() {
        assert_eq!(
            parse(&generate_tokens(
                r#"
try (
  int i = 1;
  int a = 2
) {
    i++;
} catch (Exception | Exception2 e) {
    throw e;
} catch (final Exp e) {
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
                        TryResource::Declarator(StandaloneVariableDeclarator {
                            modifiers: vec![],
                            tpe: RefCell::new(Type::Primitive(PrimitiveType {
                                name: span(2, 3, "int"),
                                tpe: PrimitiveTypeType::Int
                            })),
                            name: span(2, 7, "i"),
                            expr_opt: Some(Expr::Int(Int {
                                value: span(2, 11, "1")
                            }))
                        }),
                        TryResource::Declarator(StandaloneVariableDeclarator {
                            modifiers: vec![],
                            tpe: RefCell::new(Type::Primitive(PrimitiveType {
                                name: span(3, 3, "int"),
                                tpe: PrimitiveTypeType::Int
                            })),
                            name: span(3, 7, "a"),
                            expr_opt: Some(Expr::Int(Int {
                                value: span(3, 11, "2")
                            }))
                        }),
                    ],
                    catches: vec![
                        Catch {
                            modifiers: vec![],
                            param_name: span(6, 33, "e"),
                            class_types: vec![
                                ClassType {
                                    prefix_opt: None,
                                    name: span(6, 10, "Exception"),
                                    type_args_opt: None,
                                    def_opt: None
                                },
                                ClassType {
                                    prefix_opt: None,
                                    name: span(6, 22, "Exception2"),
                                    type_args_opt: None,
                                    def_opt: None
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
                            modifiers: vec![Modifier::Keyword(Keyword {
                                name: span(8, 10, "final")
                            })],
                            param_name: span(8, 20, "e"),
                            class_types: vec![ClassType {
                                prefix_opt: None,
                                name: span(8, 16, "Exp"),
                                type_args_opt: None,
                                def_opt: None
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
