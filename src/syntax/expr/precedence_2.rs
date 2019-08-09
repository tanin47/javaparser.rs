use nom::branch::alt;
use nom::combinator::opt;
use nom::error::ErrorKind;
use nom::{FindSubstring, IResult};
use syntax::expr::atom::{method_call, name};
use syntax::expr::{atom, precedence_2, precedence_3};
use syntax::tree::{BinaryOperation, Expr, Span, Ternary};
use syntax::{comment, expr, tag};

pub fn parse(input: Span) -> IResult<Span, Expr> {
    let (input, cond) = precedence_3::parse(input)?;
    parse_tail(cond, input)
}

pub fn parse_tail<'a>(left: Expr<'a>, input: Span<'a>) -> IResult<Span<'a>, Expr<'a>> {
    let (input, _) = match tag("?")(input) {
        Ok(ok) => ok,
        Err(_) => return precedence_3::parse_tail(left, input),
    };
    let (input, true_expr) = expr::parse(input)?;

    let (input, _) = tag(":")(input)?;
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
    use syntax::tree::{
        BinaryOperation, ClassType, Expr, Int, LiteralString, Method, MethodCall, Name, ReturnStmt,
        Ternary, TypeArg,
    };
    use test_common::{code, span};

    use super::parse;

    #[test]
    fn test_multi() {
        assert_eq!(
            parse(code(
                r#"
a ? 1 ? 2 : 3 : 4
            "#
                .trim()
            )),
            Ok((
                span(1, 18, ""),
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
