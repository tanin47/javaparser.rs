use parse::combinator::identifier;
use parse::tree::{PrimitiveType, Type, Void};
use parse::{ParseResult, Tokens};

pub mod array;
pub mod class;
pub mod primitive;
pub mod type_args;

pub fn parse(input: Tokens) -> ParseResult<Type> {
    if let Ok((input, tpe)) = primitive::parse(input) {
        Ok((input, tpe))
    } else if let Ok((input, tpe)) = class::parse(input) {
        Ok((input, tpe))
    } else {
        Err(input)
    }
}

pub fn parse_no_array(input: Tokens) -> ParseResult<Type> {
    let (input, name) = identifier(input)?;

    if name.fragment == "void" {
        Ok((input, Type::Void(Void { span: name })))
    } else if primitive::valid(name.fragment) {
        Ok((input, Type::Primitive(PrimitiveType { name })))
    } else {
        let (input, class) = class::parse_tail(input, name, None)?;
        Ok((input, Type::Class(class)))
    }
}
