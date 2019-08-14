use parse::combinator::{keyword, symbol};
use parse::tree::{Statement, Throw};
use parse::{expr, ParseResult, Tokens};

pub fn parse(input: Tokens) -> ParseResult<Statement> {
    let (input, _) = keyword("throw")(input)?;

    let (input, expr) = expr::parse(input)?;

    let (input, _) = symbol(';')(input)?;

    Ok((input, Statement::Throw(Throw { expr })))
}

#[cfg(test)]
mod tests {
    use super::parse;
    use parse::tree::{ClassType, Expr, NewObject, Statement, Throw};
    use parse::Tokens;
    use test_common::{code, span};

    #[test]
    fn test_throw() {
        assert_eq!(
            parse(&code(
                r#"
throw new Exception();
            "#
            )),
            Ok((
                &[] as Tokens,
                Statement::Throw(Throw {
                    expr: Expr::NewObject(NewObject {
                        prefix_opt: None,
                        tpe: ClassType {
                            prefix_opt: None,
                            name: span(1, 11, "Exception"),
                            type_args_opt: None
                        },
                        constructor_type_args_opt: None,
                        args: vec![],
                        body_opt: None
                    })
                })
            ))
        );
    }
}
