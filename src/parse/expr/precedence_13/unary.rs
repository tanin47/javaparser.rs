use parse::combinator::symbol;
use parse::expr::precedence_13;
use parse::tree::{Expr, UnaryOperation};
use parse::{ParseResult, Tokens};
use tokenize::span::Span;

fn op(input: Tokens) -> ParseResult<Span> {
    if let Ok(ok) = symbol('+')(input) {
        Ok(ok)
    } else if let Ok(ok) = symbol('-')(input) {
        Ok(ok)
    } else if let Ok(ok) = symbol('!')(input) {
        Ok(ok)
    } else if let Ok(ok) = symbol('~')(input) {
        Ok(ok)
    } else {
        Err(input)
    }
}

pub fn parse(input: Tokens) -> ParseResult<Expr> {
    let (input, operator) = op(input)?;

    let (input, expr) = precedence_13::parse(input)?;

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

    #[test]
    fn test_multi() {
        assert_eq!(
            parse(&code(
                r#"
+-a
            "#
            )),
            Ok((
                span(1, 4, ""),
                Expr::UnaryOperation(UnaryOperation {
                    expr: Box::new(Expr::UnaryOperation(UnaryOperation {
                        expr: Box::new(Expr::Name(Name {
                            name: span(1, 3, "a")
                        })),
                        operator: span(1, 2, "-"),
                        is_post: false
                    })),
                    operator: span(1, 1, "+"),
                    is_post: false
                })
            ))
        );
    }
}
