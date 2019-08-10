use nom::branch::alt;
use nom::IResult;

use either::Either;
use nom::error::ErrorKind;
use nom::sequence::{preceded, tuple};
use syntax::tpe::type_args;
use syntax::tree::{
    Boolean, Expr, Keyword, MethodCall, Name, NewArray, Null, PrimitiveType, Span, Type,
};
use syntax::{comment, tag, tpe};
use test_common::primitive;

pub mod array_access;
pub mod array_initializer;
pub mod lambda;
pub mod literal_char;
pub mod method_call;
pub mod name;
pub mod new_array;
pub mod new_object;
pub mod number;
pub mod parenthesized;
pub mod string;

pub fn parse(input: Span) -> IResult<Span, Expr> {
    alt((
        number::parse,
        string::parse,
        literal_char::parse,
        array_initializer::parse,
        parse_prefix_identifier,
        parse_lambda_or_parenthesized,
    ))(input)
}

fn parse_lambda_or_parenthesized(original: Span) -> IResult<Span, Expr> {
    let (input, _) = tag("(")(original)?;

    if let Ok((input, _)) = tuple((tag(")"), tag("->")))(input) {
        return lambda::parse(original);
    }

    let (input, identifier) = match name::identifier(input) {
        Ok(ok) => ok,
        Err(_) => return parenthesized::parse(original),
    };

    if tpe::primitive::valid(identifier.fragment) {
        return lambda::parse(original);
    }

    // a single unknown type param name
    if let Ok((input, _)) = tag(")")(input) {
        if let Ok(_) = tag("->")(input) {
            return lambda::parse(original);
        }
    }

    // a param name with type
    if let Ok((_, Either::Right(Name { name: _ }))) = name::parse(input) {
        return lambda::parse(original);
    }

    // Unknown type first param
    if let Ok((input, _)) = tag(",")(input) {
        return lambda::parse(original);
    }

    // The first param has typed with type arg
    if let Ok((input, Some(_))) = tpe::type_args::parse(input) {
        if let Ok((_, Either::Right(Name { name: _ }))) = name::parse(input) {
            return lambda::parse(original);
        }
    }

    parenthesized::parse(original)
}

fn parse_new_object_or_array(input: Span) -> IResult<Span, Expr> {
    if let Ok((input, Some(type_args))) = type_args::parse(input) {
        return new_object::parse_tail2(input, Some(type_args));
    }

    let (input, tpe) = tpe::parse_no_array(input)?;
    // TODO: handle this.
    let copied = tpe.clone();

    if let Ok((input, expr)) = new_array::parse_tail(input, tpe) {
        Ok((input, expr))
    } else {
        match copied {
            Type::Class(class) => new_object::parse_tail3(input, None, class),
            _ => Err(nom::Err::Error((input, ErrorKind::Tag))),
        }
    }
}

pub fn parse_prefix_identifier(original: Span) -> IResult<Span, Expr> {
    let (input, keyword_or_name) = name::parse(original)?;

    match keyword_or_name {
        Either::Left(keyword) => match keyword.name.fragment {
            "true" | "false" => Ok((
                input,
                Expr::Boolean(Boolean {
                    value: keyword.name,
                }),
            )),
            "null" => Ok((
                input,
                Expr::Null(Null {
                    value: keyword.name,
                }),
            )),
            "new" => parse_new_object_or_array(input),
            _ => Err(nom::Err::Error((input, ErrorKind::Tag))),
        },
        Either::Right(name) => {
            if let Ok(_) = tag("->")(input) {
                lambda::parse(original)
            } else if let Ok((input, args)) = method_call::parse_args(input) {
                Ok((
                    input,
                    Expr::MethodCall(MethodCall {
                        prefix_opt: None,
                        name: name.name,
                        type_args_opt: None,
                        args,
                    }),
                ))
            } else {
                array_access::parse_tail(input, Expr::Name(name))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use syntax::tree::{Expr, Int, LiteralString, Method, ReturnStmt};
    use test_common::{code, span};

    use super::parse;

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
