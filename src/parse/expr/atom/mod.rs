use either::Either;
use parse::combinator::{identifier, symbol, symbol2};
use parse::tpe::{primitive, type_args};
use parse::tree::{Boolean, Expr, Keyword, MethodCall, Name, Null, Super, This, Type};
use parse::{tpe, ParseResult, Tokens};

pub mod array_access;
pub mod array_initializer;
pub mod constructor_call;
pub mod invocation;
pub mod lambda;
pub mod literal_char;
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
    } else if let Ok(ok) = constructor_call::parse(input) {
        Ok(ok)
    } else if let Ok(ok) = parse_prefix_keyword_or_identifier(input) {
        Ok(ok)
    } else if let Ok(ok) = parse_lambda_or_parenthesized(input) {
        Ok(ok)
    } else {
        Err(input)
    }
}

fn parse_lambda_or_parenthesized(original: Tokens) -> ParseResult<Expr> {
    let (input, _) = symbol('(')(original)?;

    if let Ok((input, _)) = symbol(')')(input) {
        if let Ok((input, _)) = symbol2('-', '>')(input) {
            return lambda::parse(original);
        }
    }

    if let Ok(_) = primitive::parse(input) {
        return lambda::parse(original);
    }

    let (input, ident) = match identifier(input) {
        Ok(ok) => ok,
        Err(_) => return parenthesized::parse(original),
    };

    // a single unknown type param name
    if let Ok((input, _)) = symbol(')')(input) {
        if let Ok(_) = symbol2('-', '>')(input) {
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
        return new_object::parse_tail2(None, input, Some(type_args));
    }

    let (input, tpe) = tpe::parse_no_array(input)?;
    let copied = tpe.clone();

    if let Ok((input, expr)) = new_array::parse_tail(input, tpe) {
        Ok((input, expr))
    } else {
        match copied {
            Type::Class(class) => new_object::parse_tail3(None, input, None, class),
            _ => Err(input),
        }
    }
}

fn parse_prefix_keyword_or_identifier(original: Tokens) -> ParseResult<Expr> {
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
            "this" => Ok((
                input,
                Expr::This(This {
                    tpe_opt: None,
                    span: keyword.name,
                }),
            )),
            "super" => Ok((
                input,
                Expr::Super(Super {
                    tpe_opt: None,
                    span: keyword.name,
                }),
            )),
            _ => Err(input),
        },
        Either::Right(name) => {
            if let Ok(_) = symbol2('-', '>')(input) {
                lambda::parse(original)
            } else if let Ok((input, args)) = invocation::parse_args(input) {
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
