use parse::combinator::{get_and_not_followed_by, identifier, opt, symbol};
use parse::tree::{Labeled, Statement};
use parse::{ParseResult, Tokens};
use tokenize::span::Span;

pub mod assert;
pub mod block;
pub mod break_stmt;
pub mod class;
pub mod continue_stmt;
pub mod do_while;
pub mod expr;
pub mod for_loop;
pub mod if_else;
pub mod return_stmt;
pub mod switch;
pub mod synchronized;
pub mod throw;
pub mod try;
pub mod variable_declarators;
pub mod while_loop;

fn parse_label(input: Tokens) -> ParseResult<Span> {
    let (input, label) = identifier(input)?;

    if label.fragment == "default" {
        Err(input)
    } else {
        let (input, _) = get_and_not_followed_by(symbol(':'), symbol(':'))(input)?;

        Ok((input, label))
    }
}

fn parse_statement(input: Tokens) -> ParseResult<Statement> {
    if let Ok((input, _)) = symbol(';')(input) {
        Ok((input, Statement::Empty))
    } else if let Ok(ok) = assert::parse(input) {
        Ok(ok)
    } else if let Ok(ok) = break_stmt::parse(input) {
        Ok(ok)
    } else if let Ok(ok) = class::parse(input) {
        Ok(ok)
    } else if let Ok(ok) = continue_stmt::parse(input) {
        Ok(ok)
    } else if let Ok(ok) = return_stmt::parse(input) {
        Ok(ok)
    } else if let Ok(ok) = throw::parse(input) {
        Ok(ok)
    } else if let Ok(ok) = try::parse(input) {
        Ok(ok)
    } else if let Ok(ok) = for_loop::parse(input) {
        Ok(ok)
    } else if let Ok(ok) = do_while::parse(input) {
        Ok(ok)
    } else if let Ok(ok) = while_loop::parse(input) {
        Ok(ok)
    } else if let Ok(ok) = switch::parse(input) {
        Ok(ok)
    } else if let Ok(ok) = synchronized::parse(input) {
        Ok(ok)
    } else if let Ok(ok) = if_else::parse(input) {
        Ok(ok)
    } else if let Ok(ok) = block::parse(input) {
        Ok(ok)
    } else if let Ok(ok) = variable_declarators::parse(input) {
        Ok(ok)
    } else if let Ok(ok) = expr::parse(input) {
        Ok(ok)
    } else {
        Err(input)
    }
}

pub fn parse(input: Tokens) -> ParseResult<Statement> {
    let (input, label_opt) = opt(parse_label)(input)?;
    let (input, statement) = parse_statement(input)?;

    if let Some(label) = label_opt {
        Ok((
            input,
            Statement::Labeled(Labeled {
                label,
                statement: Box::new(statement),
            }),
        ))
    } else {
        Ok((input, statement))
    }
}

#[cfg(test)]
mod tests {
    use test_common::{code, span};

    use super::parse;
    use parse::tree::{
        ArrayType, ClassType, Expr, Labeled, Name, NewArray, PrimitiveType, ReturnStmt, Statement,
        Type, VariableDeclarator, VariableDeclarators,
    };
    use parse::Tokens;

    #[test]
    fn test_empty() {
        assert_eq!(
            parse(&code(
                r#"
;
            "#
            )),
            Ok((&[] as Tokens, Statement::Empty))
        );
    }

    #[test]
    fn test_return() {
        assert_eq!(
            parse(&code(
                r#"
return new Segment[ssize];
            "#
            )),
            Ok((
                &[] as Tokens,
                Statement::Return(ReturnStmt {
                    expr_opt: Some(Expr::NewArray(NewArray {
                        tpe: ArrayType {
                            tpe: Box::new(Type::Class(ClassType {
                                prefix_opt: None,
                                name: span(1, 12, "Segment"),
                                type_args_opt: None
                            })),
                            size_opt: Some(Box::new(Expr::Name(Name {
                                name: span(1, 20, "ssize")
                            })))
                        },
                        initializer_opt: None
                    }))
                })
            ))
        );
    }

    #[test]
    fn test_labeled_variable_declarator() {
        assert_eq!(
            parse(&code(
                r#"
label: int a;
            "#
            )),
            Ok((
                &[] as Tokens,
                Statement::Labeled(Labeled {
                    label: span(1, 1, "label"),
                    statement: Box::new(Statement::VariableDeclarators(VariableDeclarators {
                        modifiers: vec![],
                        declarators: vec![VariableDeclarator {
                            tpe: Type::Primitive(PrimitiveType {
                                name: span(1, 8, "int"),
                            }),
                            name: span(1, 12, "a"),
                            expr_opt: None
                        }]
                    }))
                })
            ))
        );
    }
}
