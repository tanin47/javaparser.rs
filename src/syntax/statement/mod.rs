use nom::branch::alt;
use nom::IResult;

use syntax::tag;
use syntax::tree::{Span, Statement};

pub mod block;
pub mod expr;
pub mod for_loop;
pub mod if_else;
pub mod return_stmt;
pub mod throw;
pub mod try;
pub mod variable_declarators;
pub mod while_loop;

pub fn parse(input: Span) -> IResult<Span, Statement> {
    alt((
        return_stmt::parse,
        throw::parse,
        try::parse,
        for_loop::parse,
        while_loop::parse,
        if_else::parse,
        block::parse,
        variable_declarators::parse,
        expr::parse,
    ))(input)
}

#[cfg(test)]
mod tests {
    use syntax::tree::{
        Expr, Int, LiteralString, Method, PrimitiveType, ReturnStmt, Statement, Type,
        VariableDeclarator, VariableDeclarators,
    };
    use test_common::{code, span};

    use super::parse;

    #[test]
    fn test_return() {
        assert_eq!(
            parse(code(
                r#"
return;
            "#
                .trim()
            )),
            Ok((
                span(1, 8, ""),
                Statement::Return(ReturnStmt { expr_opt: None })
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
                    annotateds: vec![],
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
