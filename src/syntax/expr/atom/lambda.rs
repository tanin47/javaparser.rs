use either::Either;
use nom::error::ErrorKind;
use nom::multi::separated_list;
use nom::IResult;
use syntax::def::param;
use syntax::expr::atom::name;
use syntax::statement::block::parse_block;
use syntax::tree::{Block, Expr, Int, Lambda, Param, ReturnStmt, Span, Statement, Type};
use syntax::{comment, expr, tag};

fn parse_block_or_single_expr(input: Span) -> IResult<Span, (Option<Block>, Option<Expr>)> {
    match parse_block(input) {
        Ok((input, block)) => Ok((input, (Some(block), None))),
        Err(_) => {
            let (input, expr) = expr::parse(input)?;
            Ok((input, (None, Some(expr))))
        }
    }
}

fn parse_param_with_type_or_without_type(input: Span) -> IResult<Span, Param> {
    match param::parse(input) {
        Ok(result) => Ok(result),
        Err(_) => {
            let (input, name) = name::identifier(input)?;
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

pub fn parse(input: Span) -> IResult<Span, Expr> {
    let (input, params) = if let Ok((input, _)) = tag("(")(input) {
        let (input, params) =
            separated_list(tag(","), parse_param_with_type_or_without_type)(input)?;
        let (input, _) = tag(")")(input)?;

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
        return Err(nom::Err::Error((input, ErrorKind::Tag)));
    };

    let (input, _) = tag("->")(input)?;

    let (input, (block_opt, expr_opt)) = parse_block_or_single_expr(input)?;

    Ok((
        input,
        Expr::Lambda(Lambda {
            params,
            expr_opt: match expr_opt {
                Some(expr) => Some(Box::new(expr)),
                None => None,
            },
            block_opt,
        }),
    ))
}

#[cfg(test)]
mod tests {
    use super::parse;
    use syntax::tree::{
        Block, ClassType, Expr, Int, Lambda, Method, Param, PrimitiveType, ReturnStmt, Statement,
        Type,
    };
    use test_common::{code, span};

    #[test]
    fn test_single_with_args() {
        assert_eq!(
            parse(code(
                r#"
(Test t, a, int i) -> 1
            "#
                .trim()
            )),
            Ok((
                span(1, 24, ""),
                Expr::Lambda(Lambda {
                    params: vec![
                        Param {
                            modifiers: vec![],
                            tpe: Type::Class(ClassType {
                                prefix_opt: None,
                                name: span(1, 2, "Test"),
                                type_args_opt: None
                            }),
                            is_varargs: false,
                            name: span(1, 7, "t"),
                        },
                        Param {
                            modifiers: vec![],
                            tpe: Type::UnknownType,
                            is_varargs: false,
                            name: span(1, 10, "a"),
                        },
                        Param {
                            modifiers: vec![],
                            tpe: Type::Primitive(PrimitiveType {
                                name: span(1, 13, "int")
                            }),
                            is_varargs: false,
                            name: span(1, 17, "i"),
                        }
                    ],
                    expr_opt: Some(Box::new(Expr::Int(Int {
                        value: span(1, 23, "1")
                    }))),
                    block_opt: None
                })
            ))
        );
    }

    #[test]
    fn test_simple() {
        assert_eq!(
            parse(code(
                r#"
(x) -> 2
            "#
                .trim()
            )),
            Ok((
                span(1, 9, ""),
                Expr::Lambda(Lambda {
                    params: vec![Param {
                        modifiers: vec![],
                        tpe: Type::UnknownType,
                        is_varargs: false,
                        name: span(1, 2, "x"),
                    }],
                    expr_opt: Some(Box::new(Expr::Int(Int {
                        value: span(1, 8, "2")
                    }))),
                    block_opt: None
                })
            ))
        );
    }

    #[test]
    fn test_simple2() {
        assert_eq!(
            parse(code(
                r#"
x -> 2
            "#
                .trim()
            )),
            Ok((
                span(1, 7, ""),
                Expr::Lambda(Lambda {
                    params: vec![Param {
                        modifiers: vec![],
                        tpe: Type::UnknownType,
                        is_varargs: false,
                        name: span(1, 1, "x"),
                    }],
                    expr_opt: Some(Box::new(Expr::Int(Int {
                        value: span(1, 6, "2")
                    }))),
                    block_opt: None
                })
            ))
        );
    }

    #[test]
    fn test_block() {
        assert_eq!(
            parse(code(
                r#"
() -> { return 1; }
            "#
                .trim()
            )),
            Ok((
                span(1, 20, ""),
                Expr::Lambda(Lambda {
                    params: vec![],
                    expr_opt: None,
                    block_opt: Some(Block {
                        stmts: vec![Statement::Return(ReturnStmt {
                            expr_opt: Some(Expr::Int(Int {
                                value: span(1, 16, "1")
                            }))
                        })]
                    })
                })
            ))
        );
    }
}
