use parse::combinator::symbol2;
use parse::expr::{precedence_3, precedence_4};
use parse::tree::{BinaryOperation, Expr};
use parse::{ParseResult, Tokens};

pub fn parse(input: Tokens) -> ParseResult<Expr> {
    let (input, left) = precedence_4::parse(input)?;
    precedence_3::parse_tail(left, input)
}

pub fn parse_tail<'a>(left: Expr<'a>, input: Tokens<'a>) -> ParseResult<'a, Expr<'a>> {
    if let Ok((input, operator)) = symbol2('|', '|')(input) {
        let (input, right) = precedence_4::parse(input)?;

        let expr = Expr::BinaryOperation(BinaryOperation {
            left: Box::new(left),
            operator,
            right: Box::new(right),
        });

        precedence_3::parse_tail(expr, input)
    } else {
        Ok((input, left))
    }
}

#[cfg(test)]
mod tests {
    use test_common::{code, span};

    use super::parse;
    use parse::tree::{BinaryOperation, Boolean, Expr, FieldAccess, Name};
    use parse::Tokens;

    #[test]
    fn test_precedence() {
        assert_eq!(
            parse(&code(
                r#"
true || false && t.a || false
            "#
            )),
            Ok((
                &[] as Tokens,
                Expr::BinaryOperation(BinaryOperation {
                    left: Box::new(Expr::BinaryOperation(BinaryOperation {
                        left: Box::new(Expr::Boolean(Boolean {
                            value: span(1, 1, "true")
                        })),
                        operator: span(1, 6, "||"),
                        right: Box::new(Expr::BinaryOperation(BinaryOperation {
                            left: Box::new(Expr::Boolean(Boolean {
                                value: span(1, 9, "false")
                            })),
                            operator: span(1, 15, "&&"),
                            right: Box::new(Expr::FieldAccess(FieldAccess {
                                expr: Box::new(Expr::Name(Name {
                                    name: span(1, 18, "t")
                                })),
                                field: Name {
                                    name: span(1, 20, "a")
                                }
                            }))
                        })),
                    })),
                    operator: span(1, 22, "||"),
                    right: Box::new(Expr::Boolean(Boolean {
                        value: span(1, 25, "false")
                    }))
                })
            ))
        );
    }
}
