use nom::branch::alt;
use nom::character::is_digit;
use nom::combinator::opt;
use nom::error::ErrorKind;
use nom::multi::separated_list;
use nom::IResult;
use syntax::def::class_body;
use syntax::tpe::type_args;
use syntax::tree::{ArrayInitializer, ArrayType, Expr, Int, NewArray, NewObject, Span, Type};
use syntax::{comment, expr, tag, tpe};

pub fn parse_initializer(input: Span) -> IResult<Span, ArrayInitializer> {
    let (input, _) = tag("{")(input)?;

    let (input, items) = separated_list(tag(","), expr::parse)(input)?;
    let (input, _) = opt(tag(","))(input)?;

    let (input, _) = tag("}")(input)?;

    Ok((input, ArrayInitializer { items }))
}

pub fn parse(input: Span) -> IResult<Span, Expr> {
    let (input, init) = parse_initializer(input)?;
    Ok((input, Expr::ArrayInitializer(init)))
}

#[cfg(test)]
mod tests {
    use super::parse;
    use syntax::tree::{
        ArrayInitializer, Block, ClassBody, ClassType, Expr, Int, LiteralString, Method, NewArray,
        NewObject, ReturnStmt, TypeArg,
    };
    use test_common::{code, primitive, span};

    #[test]
    fn test() {
        assert_eq!(
            parse(code("{}")),
            Ok((
                span(1, 3, ""),
                Expr::ArrayInitializer(ArrayInitializer { items: vec![] })
            ))
        );
    }

    #[test]
    fn test_nested() {
        assert_eq!(
            parse(code(
                r#"
{ 1, {2}}
            "#
                .trim()
            )),
            Ok((
                span(1, 10, ""),
                Expr::ArrayInitializer(ArrayInitializer {
                    items: vec![
                        Expr::Int(Int {
                            value: span(1, 3, "1")
                        }),
                        Expr::ArrayInitializer(ArrayInitializer {
                            items: vec![Expr::Int(Int {
                                value: span(1, 7, "2")
                            }),]
                        })
                    ]
                })
            ))
        );
    }
}
