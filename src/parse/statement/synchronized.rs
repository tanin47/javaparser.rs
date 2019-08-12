use parse::combinator::{symbol, word};
use parse::statement::block;
use parse::tree::{Statement, Synchronized};
use parse::{expr, ParseResult, Tokens};

pub fn parse(input: Tokens) -> ParseResult<Statement> {
    let (input, _) = word("synchronized")(input)?;
    let (input, _) = symbol('(')(input)?;
    let (input, expr) = expr::parse(input)?;
    let (input, _) = symbol(')')(input)?;
    let (input, block) = block::parse_block(input)?;

    Ok((
        input,
        Statement::Synchronized(Synchronized {
            expr: Box::new(expr),
            block,
        }),
    ))
}
