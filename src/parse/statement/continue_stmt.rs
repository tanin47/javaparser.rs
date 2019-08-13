use parse::combinator::{identifier, opt, symbol, word};
use parse::tree::{Continue, Statement};
use parse::{expr, ParseResult, Tokens};

pub fn parse(input: Tokens) -> ParseResult<Statement> {
    let (input, span) = word("continue")(input)?;
    let (input, identifier_opt) = opt(identifier)(input)?;
    let (input, _) = symbol(';')(input)?;

    Ok((input, Statement::Continue(Continue { identifier_opt })))
}
