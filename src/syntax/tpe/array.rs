use nom::branch::alt;
use nom::bytes::complete::{tag, take_while, take_while1};
use nom::character::is_digit;
use nom::IResult;
use syntax::comment;
use syntax::tree::{ArrayType, ClassType, Expr, Int, PrimitiveType, Span, Type};

pub fn parse_tail<'a>(input: Span<'a>, tpe: Type<'a>) -> IResult<Span<'a>, Type<'a>> {
    match tag("[")(input) as IResult<Span, Span> {
        Ok((input, _)) => {
            let (input, _) = comment::parse(input)?;
            let (input, _) = tag("]")(input)?;
            parse_tail(
                input,
                Type::Array(ArrayType {
                    tpe: Box::new(tpe),
                    size_opt: None,
                }),
            )
        }
        Err(_) => Ok((input, tpe)),
    }
}
