use parse::combinator::{identifier, opt, symbol, word};
use parse::tree::{Break, Statement};
use parse::{expr, ParseResult, Tokens};

pub fn parse(input: Tokens) -> ParseResult<Statement> {
    let (input, span) = word("break")(input)?;
    let (input, identifier_opt) = opt(identifier)(input)?;
    let (input, _) = symbol(';')(input)?;

    Ok((input, Statement::Break(Break { identifier_opt })))
}
