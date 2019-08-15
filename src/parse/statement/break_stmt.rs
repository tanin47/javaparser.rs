use parse::combinator::{identifier, keyword, opt, symbol};
use parse::tree::{Break, Statement};
use parse::{expr, ParseResult, Tokens};

pub fn parse(input: Tokens) -> ParseResult<Statement> {
    let (input, span) = keyword("break")(input)?;
    let (input, identifier_opt) = opt(identifier)(input)?;
    let (input, _) = symbol(';')(input)?;

    Ok((input, Statement::Break(Break { identifier_opt })))
}
