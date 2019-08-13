use parse::combinator::keyword;
use parse::tree::{Type, Void};
use parse::{ParseResult, Tokens};

pub fn parse(input: Tokens) -> ParseResult<Void> {
    let (input, span) = keyword("void")(input)?;
    Ok((input, Void { span }))
}
