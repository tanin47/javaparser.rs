use parse::combinator::{any_keyword, identifier};
use parse::tree::{PrimitiveType, Type, Void};
use parse::{ParseResult, Tokens};

pub mod array;
pub mod class;
pub mod primitive;
pub mod reference;
pub mod type_args;
pub mod void;

pub fn parse(input: Tokens) -> ParseResult<Type> {
    if let Ok((input, tpe)) = void::parse(input) {
        Ok((input, Type::Void(tpe)))
    } else if let Ok(ok) = primitive::parse(input) {
        Ok(ok)
    } else if let Ok(ok) = class::parse(input) {
        Ok(ok)
    } else {
        Err(input)
    }
}

pub fn parse_no_array(input: Tokens) -> ParseResult<Type> {
    if let Ok((input, tpe)) = void::parse(input) {
        Ok((input, Type::Void(tpe)))
    } else if let Ok((input, tpe)) = primitive::parse_no_array(input) {
        Ok((input, Type::Primitive(tpe)))
    } else if let Ok((input, tpe)) = class::parse_no_array(input) {
        Ok((input, Type::Class(tpe)))
    } else {
        Err(input)
    }
}
