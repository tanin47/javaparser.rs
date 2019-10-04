use parse::combinator::{identifier, keyword, opt, symbol};
use parse::tree::{Continue, Statement};
use parse::{expr, ParseResult, Tokens};

pub fn parse<'def, 'r>(input: Tokens<'def, 'r>) -> ParseResult<'def, 'r, Statement<'def>> {
    let (input, span) = keyword("continue")(input)?;
    let (input, identifier_opt) = opt(identifier)(input)?;
    let (input, _) = symbol(';')(input)?;

    Ok((input, Statement::Continue(Continue { identifier_opt })))
}
