use nom::branch::alt;
use nom::bytes::complete::{tag, take_while, take_while1};
use nom::character::is_digit;
use nom::combinator::{map, opt};
use nom::error::ErrorKind;
use nom::{FindSubstring, IResult};
use syntax::expr::atom;
use syntax::expr::atom::{method_call, name, parenthesized};
use syntax::tree::{ArrayAccess, Expr, Int, Span};
use syntax::{comment, expr};

pub fn parse_index(input: Span) -> IResult<Span, Expr> {
    let (input, _) = comment::parse(input)?;
    let (input, _) = tag("[")(input)?;

    let (input, index) = expr::parse(input)?;

    let (input, _) = comment::parse(input)?;
    let (input, _) = tag("]")(input)?;

    Ok((input, index))
}

pub fn parse_tail<'a>(input: Span<'a>, expr: Expr<'a>) -> IResult<Span<'a>, Expr<'a>> {
    let (input, index_opt) = opt(parse_index)(input)?;

    match index_opt {
        Some(index) => parse_tail(
            input,
            Expr::ArrayAccess(ArrayAccess {
                expr: Box::new(expr),
                index: Box::new(index),
            }),
        ),
        None => Ok((input, expr)),
    }
}

//#[cfg(test)]
//mod tests {
//    use syntax::tree::{
//        ArrayAccess, ClassType, Expr, Int, LiteralString, Method, MethodCall, Name, ReturnStmt,
//        TypeArg,
//    };
//    use test_common::{code, span};
//
//    use super::parse;
//
//    #[test]
//    fn test_multi() {
//        assert_eq!(
//            parse(code(
//                r#"
//abc[1][2]
//            "#
//                .trim()
//            )),
//            Ok((
//                span(1, 10, ""),
//                Expr::ArrayAccess(ArrayAccess {
//                    expr: Box::new(Expr::ArrayAccess(ArrayAccess {
//                        expr: Box::new(Expr::Name(Name {
//                            name: span(1, 1, "abc")
//                        })),
//                        index: Box::new(Expr::Int(Int {
//                            value: span(1, 5, "1")
//                        }))
//                    })),
//                    index: Box::new(Expr::Int(Int {
//                        value: span(1, 8, "2")
//                    }))
//                })
//            ))
//        );
//    }
//}
