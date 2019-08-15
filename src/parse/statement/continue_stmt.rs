use parse::combinator::{identifier, keyword, opt, symbol};
use parse::tree::{Continue, Statement};
use parse::{expr, ParseResult, Tokens};

pub fn parse(input: Tokens) -> ParseResult<Statement> {
    let (input, span) = keyword("continue")(input)?;
    let (input, identifier_opt) = opt(identifier)(input)?;
    let (input, _) = symbol(';')(input)?;

    Ok((input, Statement::Continue(Continue { identifier_opt })))
}
