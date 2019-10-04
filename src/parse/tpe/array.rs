use parse::combinator::symbol;
use parse::tree::{ArrayType, Type};
use parse::{ParseResult, Tokens};

pub fn parse_tail<'def, 'r>(
    input: Tokens<'def, 'r>,
    tpe: Type<'def>,
) -> ParseResult<'def, 'r, Type<'def>> {
    if let Ok((input, _)) = symbol('[')(input) {
        let (input, _) = symbol(']')(input)?;
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
