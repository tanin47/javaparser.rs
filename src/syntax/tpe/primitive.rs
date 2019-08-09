use nom::branch::alt;
use nom::bytes::complete::{tag, take_while, take_while1};
use nom::character::is_digit;
use nom::error::ErrorKind;
use nom::IResult;
use syntax::comment;
use syntax::expr::atom::name;
use syntax::tpe::array;
use syntax::tree::{Expr, Int, PrimitiveType, Span, Type};

pub fn valid(s: &str) -> bool {
    s == "boolean"
        || s == "byte"
        || s == "short"
        || s == "int"
        || s == "long"
        || s == "float"
        || s == "double"
        || s == "char"
        || s == "void"
}

pub fn parse_no_array(input: Span) -> IResult<Span, PrimitiveType> {
    let (original, _) = comment::parse(input)?;
    let (input, name) = name::identifier(original)?;

    if valid(name.fragment) {
        Ok((input, PrimitiveType { name }))
    } else {
        Err(nom::Err::Error((original, ErrorKind::Tag)))
    }
}

pub fn parse(input: Span) -> IResult<Span, Type> {
    let (input, tpe) = parse_no_array(input)?;
    array::parse_tail(input, Type::Primitive(tpe))
}

#[cfg(test)]
mod tests {
    use super::parse;
    use syntax::tree::{ArrayType, Expr, Int, Method, PrimitiveType, ReturnStmt, Type};
    use test_common::{code, span};

    #[test]
    fn test_long() {
        assert_eq!(
            parse(code(
                r#"
long
            "#
                .trim()
            )),
            Ok((
                span(1, 5, ""),
                Type::Primitive(PrimitiveType {
                    name: span(1, 1, "long"),
                })
            ))
        );
    }

    #[test]
    fn test_array_2d() {
        assert_eq!(
            parse(code(
                r#"
long[][]
            "#
                .trim()
            )),
            Ok((
                span(1, 9, ""),
                Type::Array(ArrayType {
                    tpe: Box::new(Type::Array(ArrayType {
                        tpe: Box::new(Type::Primitive(PrimitiveType {
                            name: span(1, 1, "long"),
                        })),
                        size_opt: None
                    })),
                    size_opt: None
                })
            ))
        );
    }
}
