use parse::combinator::symbol2;
use parse::expr::precedence_14;
use parse::tree::{Expr, UnaryOperation};
use parse::{ParseResult, Tokens};
use tokenize::span::Span;

fn op(input: Tokens) -> ParseResult<Span> {
    if let Ok(ok) = symbol2('+', '+')(input) {
        Ok(ok)
    } else if let Ok(ok) = symbol2('-', '-')(input) {
        Ok(ok)
    } else {
        Err(input)
    }
}

pub fn parse(input: Tokens) -> ParseResult<Expr> {
    let (input, operator) = op(input)?;
    let (input, expr) = precedence_14::parse(input)?;

    Ok((
        input,
        Expr::UnaryOperation(UnaryOperation {
            expr: Box::new(expr),
            operator,
            is_post: false,
        }),
    ))
}

#[cfg(test)]
mod tests {
    use test_common::{code, span};

    use super::parse;
    use parse::tree::{Expr, Name, UnaryOperation};
    use parse::Tokens;

    #[test]
    fn test_increment() {
        assert_eq!(
            parse(&code(
                r#"
++abc
            "#
            )),
            Ok((
                &[] as Tokens,
                Expr::UnaryOperation(UnaryOperation {
                    expr: Box::new(Expr::Name(Name {
                        name: span(1, 3, "abc")
                    })),
                    operator: span(1, 1, "++"),
                    is_post: false
                })
            ))
        );
    }

    #[test]
    fn test_decrement() {
        assert_eq!(
            parse(&code(
                r#"
--abc
            "#
            )),
            Ok((
                &[] as Tokens,
                Expr::UnaryOperation(UnaryOperation {
                    expr: Box::new(Expr::Name(Name {
                        name: span(1, 3, "abc")
                    })),
                    operator: span(1, 1, "--"),
                    is_post: false
                })
            ))
        );
    }
}
