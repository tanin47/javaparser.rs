use parse::combinator::{identifier, keyword, opt, symbol};
use parse::tree::{Break, Statement};
use parse::{expr, ParseResult, Tokens};

pub fn parse<'def, 'r>(input: Tokens<'def, 'r>) -> ParseResult<'def, 'r, Statement<'def>> {
    let (input, span) = keyword("break")(input)?;
    let (input, identifier_opt) = opt(identifier)(input)?;
    let (input, _) = symbol(';')(input)?;

    Ok((input, Statement::Break(Break { identifier_opt })))
}
