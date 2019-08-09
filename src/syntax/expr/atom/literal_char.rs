use nom::bytes::complete::take_while;
use nom::character::is_digit;
use nom::IResult;
use syntax::tree::{Char, Expr, Int, LiteralString, Span};
use syntax::{comment, tag};

pub fn parse(input: Span) -> IResult<Span, Expr> {
    let (input, _) = comment::parse(input)?;
    let (input, opening) = tag("'")(input)?;
    let (input, value) = take_while(|x| x != '\'')(input)?;
    let (input, ending) = tag("'")(input)?;

    Ok((input, Expr::Char(Char { value })))
}

#[cfg(test)]
mod tests {
    use super::parse;
    use syntax::tree::{Char, Expr, Int, LiteralString, Method, ReturnStmt};
    use test_common::{code, span};

    #[test]
    fn test_string() {
        assert_eq!(
            parse(code(
                r#"
'a'
            "#
                .trim()
            )),
            Ok((
                span(1, 4, ""),
                Expr::Char(Char {
                    value: span(1, 2, "a")
                })
            ))
        );
    }
}
