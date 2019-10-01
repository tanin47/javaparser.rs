use parse::combinator::{get_and_not_followed_by, keyword, symbol, symbol2};
use parse::expr::precedence_10;
use parse::tree::{BinaryOperation, Expr, InstanceOf};
use parse::{tpe, ParseResult, Tokens};
use tokenize::span::Span;

fn op(input: Tokens) -> ParseResult<Span> {
    if let Ok(ok) = symbol2('<', '=')(input) {
        Ok(ok)
    } else if let Ok(ok) = symbol2('>', '=')(input) {
        Ok(ok)
    } else if let Ok(ok) = get_and_not_followed_by(symbol('<'), symbol('<'))(input) {
        Ok(ok)
    } else if let Ok(ok) = get_and_not_followed_by(symbol('>'), symbol('>'))(input) {
        Ok(ok)
    } else if let Ok(ok) = keyword("instanceof")(input) {
        Ok(ok)
    } else {
        Err(input)
    }
}

pub fn parse_tail<'a>(left: Expr<'a>, input: Tokens<'a>) -> ParseResult<'a, Expr<'a>> {
    if let Ok((input, operator)) = op(input) {
        if operator.fragment == "instanceof" {
            let (input, tpe) = tpe::parse(input)?;

            Ok((
                input,
                Expr::InstanceOf(InstanceOf {
                    expr: Box::new(left),
                    operator,
                    tpe,
                }),
            ))
        } else {
            let (input, right) = precedence_10::parse(input)?;

            let expr = Expr::BinaryOperation(BinaryOperation {
                left: Box::new(left),
                operator,
                right: Box::new(right),
            });

            parse_tail(expr, input)
        }
    } else {
        Ok((input, left))
    }
}

pub fn parse(input: Tokens) -> ParseResult<Expr> {
    let (input, left) = precedence_10::parse(input)?;
    parse_tail(left, input)
}

#[cfg(test)]
mod tests {
    use test_common::{code, span};

    use super::parse;
    use parse::tree::{ClassType, Expr, InstanceOf, Name, Type};
    use parse::Tokens;

    #[test]
    fn test_instanceof() {
        assert_eq!(
            parse(&code(
                r#"
a instanceof Class
            "#
            )),
            Ok((
                &[] as Tokens,
                Expr::InstanceOf(InstanceOf {
                    expr: Box::new(Expr::Name(Name {
                        name: span(1, 1, "a")
                    })),
                    operator: span(1, 3, "instanceof"),
                    tpe: Type::Class(ClassType {
                        prefix_opt: None,
                        name: span(1, 14, "Class"),
                        type_args_opt: None,
                        def_opt: None
                    })
                })
            ))
        );
    }
}
