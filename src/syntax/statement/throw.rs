use nom::character::complete::multispace0;
use nom::character::is_space;
use nom::IResult;

use syntax::tree::{Class, Method, Statement, Throw};
use syntax::tree::{ReturnStmt, Span};
use syntax::{comment, expr, tag};

pub fn parse(input: Span) -> IResult<Span, Statement> {
    let (input, _) = tag("throw")(input)?;

    let (input, expr) = expr::parse(input)?;

    let (input, _) = tag(";")(input)?;

    Ok((input, Statement::Throw(Throw { expr })))
}

#[cfg(test)]
mod tests {
    use super::parse;
    use syntax::tree::{
        ClassType, Expr, LiteralString, Method, NewObject, ReturnStmt, Statement, Throw,
    };
    use test_common::{code, span};

    #[test]
    fn test_throw() {
        assert_eq!(
            parse(code(
                r#"
throw new Exception();
            "#
                .trim()
            )),
            Ok((
                span(1, 23, ""),
                Statement::Throw(Throw {
                    expr: Expr::NewObject(NewObject {
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
