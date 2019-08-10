use nom::branch::alt;
use nom::character::complete::{alphanumeric0, digit0, digit1};
use nom::character::is_digit;
use nom::combinator::{map, opt};
use nom::error::ErrorKind;
use nom::multi::separated_list;
use nom::IResult;
use syntax::def::class_body;
use syntax::expr::atom::array_initializer;
use syntax::tpe::type_args;
use syntax::tree::{
    ArrayInitializer, ArrayType, ClassType, Expr, Int, NewArray, NewObject, Span, Type,
};
use syntax::{comment, expr, tag, tpe};

fn parse_array_brackets<'a>(input: Span<'a>, tpe: Type<'a>) -> IResult<Span<'a>, Type<'a>> {
    let (input, _) = match tag("[")(input) as IResult<Span, Span> {
        Ok(result) => result,
        Err(_) => return Ok((input, tpe)),
    };

    let (input, size_opt) = if let Ok((input, _)) = tag("]")(input) {
        (input, None)
    } else {
        let (input, size) = expr::parse(input)?;
        let (input, _) = tag("]")(input)?;
        (input, Some(Box::new(size)))
    };

    let (input, inner) = parse_array_brackets(input, tpe)?;

    Ok((
        input,
        Type::Array(ArrayType {
            tpe: Box::new(inner),
            size_opt,
        }),
    ))
}

pub fn parse_tail<'a>(input: Span<'a>, tpe: Type<'a>) -> IResult<Span<'a>, Expr<'a>> {
    let (input, tpe) = match parse_array_brackets(input, tpe) {
        Ok((input, Type::Array(array))) => (input, array),
        other => return Err(nom::Err::Error((input, ErrorKind::Tag))),
    };
    let (input, _) = comment::parse(input)?;
    let (input, initializer_opt) = opt(array_initializer::parse_initializer)(input)?;

    Ok((
        input,
        Expr::NewArray(NewArray {
            tpe,
            initializer_opt,
        }),
    ))
}

pub fn parse(input: Span) -> IResult<Span, Expr> {
    let (input, _) = comment::parse(input)?;
    let (input, t) = tag("new")(input)?;

    let (input, _) = comment::parse(input)?;
    let (input, tpe) = tpe::parse_no_array(input)?;

    parse_tail(input, tpe)
}

#[cfg(test)]
mod tests {
    use super::parse;
    use syntax::tree::{
        ArrayInitializer, ArrayType, Block, ClassBody, ClassType, Expr, Int, LiteralString, Method,
        Name, NewArray, NewObject, PrimitiveType, ReturnStmt, Type, TypeArg,
    };
    use test_common::{code, primitive, span};

    #[test]
    fn test_array_class() {
        assert_eq!(
            parse(code(
                r#"
new Test[size]
            "#
                .trim()
            )),
            Ok((
                span(1, 15, ""),
                Expr::NewArray(NewArray {
                    tpe: ArrayType {
                        tpe: Box::new(Type::Class(ClassType {
                            prefix_opt: None,
                            name: span(1, 5, "Test"),
                            type_args_opt: None
                        })),
                        size_opt: Some(Box::new(Expr::Name(Name {
                            name: span(1, 10, "size")
                        })))
                    },
                    initializer_opt: None
                })
            ))
        );
    }

    #[test]
    fn test_array_primitive() {
        assert_eq!(
            parse(code(
                r#"
new int[2][]
            "#
                .trim()
            )),
            Ok((
                span(1, 13, ""),
                Expr::NewArray(NewArray {
                    tpe: ArrayType {
                        tpe: Box::new(Type::Array(ArrayType {
                            tpe: Box::new(Type::Primitive(PrimitiveType {
                                name: span(1, 5, "int")
                            })),
                            size_opt: None
                        })),
                        size_opt: Some(Box::new(Expr::Int(Int {
                            value: span(1, 9, "2")
                        })))
                    },
                    initializer_opt: None
                })
            ))
        );
    }

    #[test]
    fn test_initializer() {
        assert_eq!(
            parse(code(
                r#"
new int[] { 1, {2}}
            "#
                .trim()
            )),
            Ok((
                span(1, 20, ""),
                Expr::NewArray(NewArray {
                    tpe: ArrayType {
                        tpe: Box::new(Type::Primitive(PrimitiveType {
                            name: span(1, 5, "int")
                        })),
                        size_opt: None
                    },
                    initializer_opt: Some(ArrayInitializer {
                        items: vec![
                            Expr::Int(Int {
                                value: span(1, 13, "1")
                            }),
                            Expr::ArrayInitializer(ArrayInitializer {
                                items: vec![Expr::Int(Int {
                                    value: span(1, 17, "2")
                                }),]
                            })
                        ]
                    })
                })
            ))
        );
    }
}
