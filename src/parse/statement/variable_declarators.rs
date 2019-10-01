use parse::combinator::{identifier, separated_nonempty_list, symbol};
use parse::def::modifiers;
use parse::tree::{
    StandaloneVariableDeclarator, Statement, Type, VariableDeclarator, VariableDeclarators,
};
use parse::{expr, tpe, ParseResult, Tokens};
use std::cell::RefCell;

pub fn parse_single<'a>(
    original_tpe: Type<'a>,
) -> impl Fn(Tokens<'a>) -> ParseResult<'a, VariableDeclarator<'a>> {
    move |input: Tokens<'a>| {
        let (input, name) = identifier(input)?;

        let (input, tpe) = tpe::array::parse_tail(input, original_tpe.clone())?;

        let (input, expr_opt) = match symbol('=')(input) {
            Ok((input, _)) => {
                let (input, expr) = expr::parse(input)?;
                (input, Some(expr))
            }
            Err(_) => (input, None),
        };

        Ok((
            input,
            VariableDeclarator {
                tpe: RefCell::new(tpe),
                name,
                expr_opt,
            },
        ))
    }
}

pub fn parse_standalone(input: Tokens) -> ParseResult<StandaloneVariableDeclarator> {
    let (input, modifiers) = modifiers::parse(input)?;
    let (input, tpe) = tpe::parse(input)?;
    let (input, declarator) = parse_single(tpe)(input)?;

    Ok((
        input,
        StandaloneVariableDeclarator {
            modifiers,
            tpe: declarator.tpe,
            name: declarator.name,
            expr_opt: declarator.expr_opt,
        },
    ))
}

pub fn parse_without_semicolon(input: Tokens) -> ParseResult<Statement> {
    let (input, modifiers) = modifiers::parse(input)?;
    let (input, tpe) = tpe::parse(input)?;

    let (input, declarators) = separated_nonempty_list(symbol(','), parse_single(tpe))(input)?;

    Ok((
        input,
        Statement::VariableDeclarators(VariableDeclarators {
            modifiers,
            declarators,
        }),
    ))
}

pub fn parse(input: Tokens) -> ParseResult<Statement> {
    let (input, declarators) = parse_without_semicolon(input)?;
    let (input, _) = symbol(';')(input)?;

    Ok((input, declarators))
}

#[cfg(test)]
mod tests {
    use super::parse;
    use parse::tree::{
        Annotated, ArrayType, ClassType, Expr, Int, MarkerAnnotated, Modifier, PrimitiveType,
        PrimitiveTypeType, Statement, Type, VariableDeclarator, VariableDeclarators,
    };
    use parse::Tokens;
    use std::cell::RefCell;
    use test_common::{code, span};

    #[test]
    fn test_bare() {
        assert_eq!(
            parse(&code(
                r#"
@Anno int a;
            "#
            )),
            Ok((
                &[] as Tokens,
                Statement::VariableDeclarators(VariableDeclarators {
                    modifiers: vec![Modifier::Annotated(Annotated::Marker(MarkerAnnotated {
                        class: ClassType {
                            prefix_opt: None,
                            name: span(1, 2, "Anno"),
                            type_args_opt: None,
                            def_opt: None
                        }
                    }))],
                    declarators: vec![VariableDeclarator {
                        tpe: RefCell::new(Type::Primitive(PrimitiveType {
                            name: span(1, 7, "int"),
                            tpe: PrimitiveTypeType::Int
                        })),
                        name: span(1, 11, "a"),
                        expr_opt: None
                    }]
                })
            ))
        );
    }

    #[test]
    fn test_weird_array() {
        assert_eq!(
            parse(&code(
                r#"
int[] a, b[];
            "#
            )),
            Ok((
                &[] as Tokens,
                Statement::VariableDeclarators(VariableDeclarators {
                    modifiers: vec![],
                    declarators: vec![
                        VariableDeclarator {
                            tpe: RefCell::new(Type::Array(ArrayType {
                                tpe: Box::new(Type::Primitive(PrimitiveType {
                                    name: span(1, 1, "int"),
                                    tpe: PrimitiveTypeType::Int
                                })),
                                size_opt: None
                            })),
                            name: span(1, 7, "a"),
                            expr_opt: None
                        },
                        VariableDeclarator {
                            tpe: RefCell::new(Type::Array(ArrayType {
                                tpe: Box::new(Type::Array(ArrayType {
                                    tpe: Box::new(Type::Primitive(PrimitiveType {
                                        name: span(1, 1, "int"),
                                        tpe: PrimitiveTypeType::Int
                                    })),
                                    size_opt: None
                                })),
                                size_opt: None
                            })),
                            name: span(1, 10, "b"),
                            expr_opt: None
                        },
                    ]
                })
            ))
        );
    }

    #[test]
    fn test_expr() {
        assert_eq!(
            parse(&code(
                r#"
int a = 1;
            "#
            )),
            Ok((
                &[] as Tokens,
                Statement::VariableDeclarators(VariableDeclarators {
                    modifiers: vec![],
                    declarators: vec![VariableDeclarator {
                        tpe: RefCell::new(Type::Primitive(PrimitiveType {
                            name: span(1, 1, "int"),
                            tpe: PrimitiveTypeType::Int
                        })),
                        name: span(1, 5, "a"),
                        expr_opt: Some(Expr::Int(Int {
                            value: span(1, 9, "1")
                        }))
                    }]
                })
            ))
        );
    }

    #[test]
    fn test_multiple() {
        assert_eq!(
            parse(&code(
                r#"
int a = 1, b[], c;
            "#
            )),
            Ok((
                &[] as Tokens,
                Statement::VariableDeclarators(VariableDeclarators {
                    modifiers: vec![],
                    declarators: vec![
                        VariableDeclarator {
                            tpe: RefCell::new(Type::Primitive(PrimitiveType {
                                name: span(1, 1, "int"),
                                tpe: PrimitiveTypeType::Int
                            })),
                            name: span(1, 5, "a"),
                            expr_opt: Some(Expr::Int(Int {
                                value: span(1, 9, "1")
                            }))
                        },
                        VariableDeclarator {
                            tpe: RefCell::new(Type::Array(ArrayType {
                                tpe: Box::new(Type::Primitive(PrimitiveType {
                                    name: span(1, 1, "int"),
                                    tpe: PrimitiveTypeType::Int
                                })),
                                size_opt: None
                            })),
                            name: span(1, 12, "b"),
                            expr_opt: None
                        },
                        VariableDeclarator {
                            tpe: RefCell::new(Type::Primitive(PrimitiveType {
                                name: span(1, 1, "int"),
                                tpe: PrimitiveTypeType::Int
                            })),
                            name: span(1, 17, "c"),
                            expr_opt: None
                        }
                    ]
                })
            ))
        );
    }
}
