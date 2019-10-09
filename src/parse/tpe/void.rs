use parse::combinator::keyword;
use parse::tree::{Type, Void};
use parse::{ParseResult, Tokens};

pub fn parse<'def, 'r>(input: Tokens<'def, 'r>) -> ParseResult<'def, 'r, Void<'def>> {
    let (input, span) = keyword("void")(input)?;
    Ok((
        input,
        Void {
            span_opt: Some(span),
        },
    ))
}
