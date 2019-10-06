use parse::combinator::{keyword, many0, symbol};
use parse::statement::block;
use parse::tree::{Case, Statement, Switch, WhileLoop};
use parse::{expr, statement, ParseResult, Tokens};

fn parse_case<'def, 'r>(input: Tokens<'def, 'r>) -> ParseResult<'def, 'r, Case<'def>> {
    let (input, label_opt) = if let Ok((input, _)) = keyword("case")(input) {
        // TODO: The below only allows EnumConstant and ConstantExpression. We could optimize something here.
        let (input, expr) = expr::parse(input)?;
        let (input, _) = symbol(':')(input)?;
        (input, Some(Box::new(expr)))
    } else if let Ok((input, _)) = keyword("default")(input) {
        let (input, _) = symbol(':')(input)?;
        (input, None)
    } else {
        return Err(input);
    };

    let (input, stmts) = many0(statement::parse)(input)?;

    Ok((input, Case { label_opt, stmts }))
}

pub fn parse<'def, 'r>(input: Tokens<'def, 'r>) -> ParseResult<'def, 'r, Statement<'def>> {
    let (input, _) = keyword("switch")(input)?;
    let (input, _) = symbol('(')(input)?;
    let (input, expr) = expr::parse(input)?;
    let (input, _) = symbol(')')(input)?;

    let (input, _) = symbol('{')(input)?;
    let (input, cases) = many0(parse_case)(input)?;
    let (input, _) = symbol('}')(input)?;

    Ok((
        input,
        Statement::Switch(Switch {
            expr: Box::new(expr),
            cases,
        }),
    ))
}

//#[cfg(test)]
//mod tests {
//    use super::parse;
//    use parse::tree::{Boolean, Case, Expr, Name, ReturnStmt, Statement, Switch};
//    use parse::Tokens;
//    use test_common::{generate_tokens, span};
//
//    #[test]
//    fn test_switch() {
//        assert_eq!(
//            parse(&generate_tokens(
//                r#"
//switch (x) {
//    case DOCUMENTATION_OUTPUT:
//        return true;
//    default:
//        return;
//}
//            "#
//            )),
//            Ok((
//                &[] as Tokens,
//                Statement::Switch(Switch {
//                    expr: Box::new(Expr::Name(Name {
//                        name: span(1, 9, "x")
//                    })),
//                    cases: vec![
//                        Case {
//                            label_opt: Some(Box::new(Expr::Name(Name {
//                                name: span(2, 10, "DOCUMENTATION_OUTPUT")
//                            }))),
//                            stmts: vec![Statement::Return(ReturnStmt {
//                                expr_opt: Some(Expr::Boolean(Boolean {
//                                    value: span(3, 16, "true")
//                                }))
//                            })]
//                        },
//                        Case {
//                            label_opt: None,
//                            stmts: vec![Statement::Return(ReturnStmt { expr_opt: None })]
//                        }
//                    ]
//                })
//            ))
//        );
//    }
//}
