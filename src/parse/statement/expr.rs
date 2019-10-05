use parse::combinator::symbol;
use parse::tree::Statement;
use parse::{expr, ParseResult, Tokens};

pub fn parse<'def, 'r>(input: Tokens<'def, 'r>) -> ParseResult<'def, 'r, Statement<'def>> {
    let (input, statement) = parse_without_semicolon(input)?;
    let (input, _) = symbol(';')(input)?;

    Ok((input, statement))
}

pub fn parse_without_semicolon<'def, 'r>(
    input: Tokens<'def, 'r>,
) -> ParseResult<'def, 'r, Statement<'def>> {
    let (input, expr) = expr::parse(input)?;
    Ok((input, Statement::Expr(expr)))
}

#[cfg(test)]
mod tests {
    use super::parse;
    use parse::tree::{
        ArrayAccess, Assigned, Assignment, Expr, FieldAccess, Int, MethodCall, Name, Statement,
    };
    use parse::Tokens;
    use std::cell::RefCell;
    use test_common::{generate_tokens, span};

    #[test]
    fn test_return_void() {
        assert_eq!(
            parse(&generate_tokens(
                r#"
a = 123;
            "#
            )),
            Ok((
                &[] as Tokens,
                Statement::Expr(Expr::Assignment(Assignment {
                    assigned: Box::new(Assigned::Name(Name {
                        name: span(1, 1, "a")
                    })),
                    operator: span(1, 3, "="),
                    expr: Box::new(Expr::Int(Int {
                        value: span(1, 5, "123")
                    }))
                }))
            ))
        );
    }

    #[test]
    fn test_complex() {
        assert_eq!(
            parse(&generate_tokens(
                r#"
a[0].b.c();
            "#
            )),
            Ok((
                &[] as Tokens,
                Statement::Expr(Expr::MethodCall(MethodCall {
                    prefix_opt: Some(Box::new(Expr::FieldAccess(FieldAccess {
                        expr: Box::new(Expr::ArrayAccess(ArrayAccess {
                            expr: Box::new(Expr::Name(Name {
                                name: span(1, 1, "a")
                            })),
                            index: Box::new(Expr::Int(Int {
                                value: span(1, 3, "0")
                            }))
                        })),
                        field: Name {
                            name: span(1, 6, "b")
                        },
                        tpe_opt: RefCell::new(None)
                    }))),
                    name: span(1, 8, "c"),
                    type_args_opt: None,
                    args: vec![]
                }))
            ))
        );
    }
}
