use nom::bytes::complete::{tag as nom_tag, take_while, take_while1};
use nom::character::complete::hex_digit1;
use nom::character::is_digit;
use nom::IResult;
use std::slice;
use syntax::comment;
use syntax::tree::{Expr, Hex, Int, Span};

pub fn parse(input: Span) -> IResult<Span, Expr> {
    let (input, _) = comment::parse(input)?;
    let (input, value) = take_while1(|x| is_digit(x as u8))(input)?;

    if let Ok((input, x)) = nom_tag("x")(input) as IResult<Span, Span> {
        let (input, tail) = hex_digit1(input)?;
        Ok((
            input,
            Expr::Hex(Hex {
                value: Span {
                    line: value.line,
                    col: value.col,
                    fragment: unsafe {
                        std::str::from_utf8(slice::from_raw_parts(
                            value.fragment.as_ptr(),
                            value.fragment.len() + x.fragment.len() + tail.fragment.len(),
                        ))
                        .unwrap()
                    },
                    extra: (),
                },
            }),
        ))
    } else {
        Ok((input, Expr::Int(Int { value })))
    }
}

#[cfg(test)]
mod tests {
    use super::parse;
    use syntax::tree::{Expr, Hex, Int, Method, ReturnStmt};
    use test_common::{code, span};

    #[test]
    fn test_int() {
        assert_eq!(
            parse(code(
                r#"
0xab1cdef123
            "#
                .trim()
            )),
            Ok((
                span(1, 13, ""),
                Expr::Hex(Hex {
                    value: span(1, 1, "0xab1cdef123")
                })
            ))
        );
    }
}
