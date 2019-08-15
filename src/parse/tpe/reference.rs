use parse::combinator::keyword;
use parse::tpe::{class, primitive};
use parse::tree::{ReferenceType, Type, Void};
use parse::{ParseResult, Tokens};

pub fn parse(input: Tokens) -> ParseResult<ReferenceType> {
    let (input, tpe) = if let Ok(ok) = primitive::parse(input) {
        ok
    } else if let Ok(ok) = class::parse(input) {
        ok
    } else {
        return Err(input);
    };

    match tpe {
        Type::Array(arr) => Ok((input, ReferenceType::Array(arr))),
        Type::Class(class) => Ok((input, ReferenceType::Class(class))),
        _ => Err(input),
    }
}
