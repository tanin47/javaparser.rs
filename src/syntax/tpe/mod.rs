use nom::branch::alt;
use nom::IResult;
use syntax::expr::atom::name;
use syntax::tree::{PrimitiveType, Span, Statement, Type, Void};

pub mod array;
pub mod class;
pub mod primitive;
pub mod type_args;

pub fn parse(input: Span) -> IResult<Span, Type> {
    alt((primitive::parse, class::parse))(input)
}

pub fn parse_no_array(input: Span) -> IResult<Span, Type> {
    let (input, name) = name::identifier(input)?;

    if name.fragment == "void" {
        Ok((input, Type::Void(Void { span: name })))
    } else if primitive::valid(name.fragment) {
        Ok((input, Type::Primitive(PrimitiveType { name })))
    } else {
        let (input, class) = class::parse_tail(input, name, None)?;
        Ok((input, Type::Class(class)))
    }
}
