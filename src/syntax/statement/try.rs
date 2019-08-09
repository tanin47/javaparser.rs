use nom::character::complete::multispace0;
use nom::character::is_space;
use nom::IResult;

use nom::combinator::opt;
use nom::multi::{many0, separated_nonempty_list};
use nom::sequence::Tuple;
use syntax::expr::atom::name;
use syntax::statement::{block, variable_declarators};
use syntax::tree::{
    Block, Catch, Class, IfElse, Method, Name, StandaloneVariableDeclarator, Statement, Try,
    VariableDeclarator,
};
use syntax::tree::{ReturnStmt, Span};
use syntax::{comment, expr, statement, tag, tpe};

fn parse_catch(input: Span) -> IResult<Span, Catch> {
    let (input, _) = tag("catch")(input)?;
    let (input, _) = tag("(")(input)?;
    let (input, class_types) =
        separated_nonempty_list(tag("|"), tpe::class::parse_no_array)(input)?;
    let (input, param_name) = name::identifier(input)?;
    let (input, _) = tag(")")(input)?;

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

fn parse_resources(input: Span) -> IResult<Span, Vec<StandaloneVariableDeclarator>> {
    let (input, _) = match tag("(")(input) {
        Ok(ok) => ok,
        Err(_) => return Ok((input, vec![])),
    };
    let (input, resources) =
        separated_nonempty_list(tag(";"), variable_declarators::parse_standalone)(input)?;
    let (input, _) = opt(tag(";"))(input)?;
    let (input, _) = tag(")")(input)?;

    Ok((input, resources))
}

pub fn parse(input: Span) -> IResult<Span, Statement> {
    let (input, _) = tag("try")(input)?;
    let (input, resources) = parse_resources(input)?;
    let (input, try) = block::parse_block(input)?;

    let (input, catches) = many0(parse_catch)(input)?;

    let (input, finally_opt) = if let Ok((input, _)) = tag("finally")(input) {
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
    use syntax::tree::{
        Block, Catch, ClassType, Expr, IfElse, Int, LiteralString, Method, MethodCall, Name,
        PrimitiveType, ReturnStmt, StandaloneVariableDeclarator, Statement, Throw, Try, Type,
        UnaryOperation,
    };
    use test_common::{code, span};

    #[test]
    fn test_if() {
        assert_eq!(
            parse(code(
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
    final();
}
            "#
                .trim()
            )),
            Ok((
                span(12, 2, ""),
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
                            annotateds: vec![],
                            tpe: Type::Primitive(PrimitiveType {
                                name: span(2, 3, "int")
                            }),
                            name: span(2, 7, "i"),
                            expr_opt: Some(Expr::Int(Int {
                                value: span(2, 11, "1")
                            }))
                        },
                        StandaloneVariableDeclarator {
                            annotateds: vec![],
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
                            name: span(11, 5, "final"),
                            type_args_opt: None,
                            args: vec![]
                        }))]
                    })
                })
            ))
        );
    }
}
