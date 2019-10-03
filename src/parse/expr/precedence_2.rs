use parse::combinator::symbol;
use parse::expr::precedence_3;
use parse::tree::{Expr, Ternary};
use parse::{expr, ParseResult, Tokens};

pub fn parse(input: Tokens) -> ParseResult<Expr> {
    let (input, cond) = precedence_3::parse(input)?;
    parse_tail(cond, input)
}

pub fn parse_tail<'a>(left: Expr<'a>, input: Tokens<'a>) -> ParseResult<'a, Expr<'a>> {
    let (input, _) = match symbol('?')(input) {
        Ok(ok) => ok,
        Err(_) => return precedence_3::parse_tail(left, input),
    };
    let (input, true_expr) = expr::parse(input)?;

    let (input, _) = symbol(':')(input)?;
    let (input, false_expr) = expr::parse(input)?;

    Ok((
        input,
        Expr::Ternary(Ternary {
            cond: Box::new(left),
            true_expr: Box::new(true_expr),
            false_expr: Box::new(false_expr),
        }),
    ))
}

#[cfg(test)]
mod tests {
    use test_common::{generate_tokens, span};

    use super::parse;
    use parse::tree::{Expr, Int, Name, Ternary};
    use parse::Tokens;

    #[test]
    fn test_multi() {
        assert_eq!(
            parse(&generate_tokens(
                r#"
a ? 1 ? 2 : 3 : 4
            "#
            )),
            Ok((
                &[] as Tokens,
                Expr::Ternary(Ternary {
                    cond: Box::new(Expr::Name(Name {
                        name: span(1, 1, "a")
                    })),
                    true_expr: Box::new(Expr::Ternary(Ternary {
                        cond: Box::new(Expr::Int(Int {
                            value: span(1, 5, "1")
                        })),
                        true_expr: Box::new(Expr::Int(Int {
                            value: span(1, 9, "2")
                        })),
                        false_expr: Box::new(Expr::Int(Int {
                            value: span(1, 13, "3")
                        }))
                    })),
                    false_expr: Box::new(Expr::Int(Int {
                        value: span(1, 17, "4")
                    }))
                })
            ))
        );
    }
}
