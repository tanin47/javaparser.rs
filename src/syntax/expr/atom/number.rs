use nom::bytes::complete::{take_while, take_while1};
use nom::character::is_digit;
use nom::IResult;
use syntax::comment;
use syntax::tree::{Expr, Int, Span};

pub fn parse(input: Span) -> IResult<Span, Expr> {
    let (input, _) = comment::parse(input)?;
    let (input, value) = take_while1(|x| is_digit(x as u8))(input)?;

    Ok((input, Expr::Int(Int { value })))
}

#[cfg(test)]
mod tests {
    use super::parse;
    use syntax::tree::{Expr, Int, Method, ReturnStmt};
    use test_common::{code, span};

    #[test]
    fn test_int() {
        assert_eq!(
            parse(code(
                r#"
123
            "#
                .trim()
            )),
            Ok((
                span(1, 4, ""),
                Expr::Int(Int {
                    value: span(1, 1, "123")
                })
            ))
        );
    }
}
