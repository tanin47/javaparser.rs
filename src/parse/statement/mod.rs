use parse::tree::Statement;
use parse::{ParseResult, Tokens};

pub mod block;
//pub mod expr;
//pub mod for_loop;
//pub mod if_else;
//pub mod return_stmt;
//pub mod synchronized;
//pub mod throw;
//pub mod try;
//pub mod variable_declarators;
//pub mod while_loop;

pub fn parse(input: Tokens) -> ParseResult<Statement> {
    //        return_stmt::parse,
    //        throw::parse,
    //        try::parse,
    //        for_loop::parse,
    //        while_loop::parse,
    //        synchronized::parse,
    //        if_else::parse,
    if let Ok(ok) = block::parse(input) {
        Ok(ok)
    //        variable_declarators::parse,
    //        expr::parse,
    } else {
        Err(input)
    }
}

#[cfg(test)]
mod tests {
    use test_common::{code, span};

    use super::parse;
    use parse::tree::{
        ArrayType, ClassType, Expr, Name, NewArray, PrimitiveType, ReturnStmt, Statement, Type,
        VariableDeclarator, VariableDeclarators,
    };
    use parse::Tokens;

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
    fn test_variable_declarator() {
        assert_eq!(
            parse(&code(
                r#"
int a;
            "#
            )),
            Ok((
                &[] as Tokens,
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
