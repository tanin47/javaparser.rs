use parse::combinator::{symbol, word};
use parse::tree::{Break, ReturnStmt, Statement};
use parse::{expr, ParseResult, Tokens};

pub fn parse(input: Tokens) -> ParseResult<Statement> {
    let (input, span) = word("break")(input)?;
    let (input, _) = symbol(';')(input)?;

    Ok((input, Statement::Break(Break { span })))
}
