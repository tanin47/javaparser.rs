use nom::bytes::complete::{tag, take_while};
use nom::character::is_digit;
use nom::IResult;
use syntax::comment;
use syntax::tree::{Expr, Int, LiteralString, Span};

pub fn parse(input: Span) -> IResult<Span, Expr> {
    let (input, _) = comment::parse(input)?;
    let (input, opening) = tag(r#"""#)(input)?;
    let (input, value) = take_while(|x| x != '"')(input)?;
    let (input, ending) = tag(r#"""#)(input)?;

    Ok((input, Expr::String(LiteralString { value })))
}

#[cfg(test)]
mod tests {
    use super::parse;
    use syntax::tree::{Expr, Int, LiteralString, Method, ReturnStmt};
    use test_common::{code, span};

    #[test]
    fn test_string() {
        assert_eq!(
            parse(code(
                r#"
"abc"
            "#
                .trim()
            )),
            Ok((
                span(1, 6, ""),
                Expr::String(LiteralString {
                    value: span(1, 2, "abc")
                })
            ))
        );
    }
}
