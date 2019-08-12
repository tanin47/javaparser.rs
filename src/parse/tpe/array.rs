use parse::combinator::symbol;
use parse::tree::{ArrayType, Type};
use parse::{ParseResult, Tokens};

pub fn parse_tail<'a>(input: Tokens<'a>, tpe: Type<'a>) -> ParseResult<'a, Type<'a>> {
    if let Ok((input, _)) = symbol("[")(input) {
        let (input, _) = symbol("]")(input)?;
        parse_tail(
            input,
            Type::Array(ArrayType {
                tpe: Box::new(tpe),
                size_opt: None,
            }),
        )
    } else {
        Ok((input, tpe))
    }
}
