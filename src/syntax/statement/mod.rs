use nom::branch::alt;
use nom::IResult;

use syntax::tree::{Span, Statement};
use syntax::{comment, tag};

pub mod block;
pub mod expr;
pub mod for_loop;
pub mod if_else;
pub mod return_stmt;
pub mod synchronized;
pub mod throw;
pub mod try;
pub mod variable_declarators;
pub mod while_loop;

pub fn parse(input: Span) -> IResult<Span, Statement> {
    let (input, _) = comment::parse(input)?;
    alt((
        return_stmt::parse,
        throw::parse,
        try::parse,
        for_loop::parse,
        while_loop::parse,
        synchronized::parse,
        if_else::parse,
        block::parse,
        variable_declarators::parse,
        expr::parse,
    ))(input)
}

#[cfg(test)]
mod tests {
    use syntax::tree::{
        ArrayType, ClassType, Expr, Int, LiteralString, Method, Name, NewArray, PrimitiveType,
        ReturnStmt, Statement, Type, VariableDeclarator, VariableDeclarators,
    };
    use test_common::{code, span};

    use super::parse;

    #[test]
    fn test_return() {
        assert_eq!(
            parse(code(
                r#"
return new Segment[ssize];
            "#
                .trim()
            )),
            Ok((
                span(1, 27, ""),
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
    fn test_variable_declarator() {
        assert_eq!(
            parse(code(
                r#"
int a;
            "#
                .trim()
            )),
            Ok((
                span(1, 7, ""),
                Statement::VariableDeclarators(VariableDeclarators {
                    modifiers: vec![],
                    declarators: vec![VariableDeclarator {
                        tpe: Type::Primitive(PrimitiveType {
                            name: span(1, 1, "int"),
                        }),
                        name: span(1, 5, "a"),
                        expr_opt: None
                    }]
                })
            ))
        );
    }
}
