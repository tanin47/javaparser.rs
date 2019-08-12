use either::Either;
use parse::tpe::type_args;
use parse::tree::{Boolean, Expr, MethodCall, Name, Null, Type};
use parse::{tpe, ParseResult, Tokens};

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

pub fn parse(input: Tokens) -> ParseResult<Expr> {
    if let Ok(ok) = number::parse(input) {
        Ok(ok)
    } else if let Ok(ok) = string::parse(input) {
        Ok(ok)
    } else if let Ok(ok) = literal_char::parse(input) {
        Ok(ok)
    } else if let Ok(ok) = array_initializer::parse(input) {
        Ok(ok)
    } else if let Ok(ok) = parse_prefix_identifier(input) {
        Ok(ok)
    } else if let Ok(ok) = parse_lambda_or_parenthesized(input) {
        Ok(ok)
    } else {
        Err(input)
    }
}

fn parse_lambda_or_parenthesized(original: Tokens) -> ParseResult<Expr> {
    let (input, _) = symbol('(')(original)?;

    if let Ok((input, _)) = tuple((symbol(')'), tag("->")))(input) {
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
    if let Ok((input, _)) = symbol(')')(input) {
        if let Ok(_) = tag("->")(input) {
            return lambda::parse(original);
        }
    }

    // a param name with type
    if let Ok((_, Either::Right(Name { name: _ }))) = name::parse(input) {
        return lambda::parse(original);
    }

    // Unknown type first param
    if let Ok((input, _)) = symbol(',')(input) {
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

fn parse_new_object_or_array(input: Tokens) -> ParseResult<Expr> {
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

pub fn parse_prefix_identifier(original: Tokens) -> ParseResult<Expr> {
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
    use test_common::{code, span};

    use super::parse;
    use parse::tree::{Expr, Int, LiteralString};

    #[test]
    fn test_string() {
        assert_eq!(
            parse(&code(
                r#"
"abc"
            "#
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
            parse(&code(
                r#"
123
            "#
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
