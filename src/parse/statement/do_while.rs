use parse::combinator::{symbol, word};
use parse::statement::block;
use parse::tree::{DoWhile, Statement, WhileLoop};
use parse::{expr, ParseResult, Tokens};

pub fn parse(input: Tokens) -> ParseResult<Statement> {
    let (input, _) = word("do")(input)?;
    let (input, block) = block::parse_block_or_single_statement(input)?;
    let (input, _) = word("while")(input)?;
    let (input, _) = symbol('(')(input)?;
    let (input, cond) = expr::parse(input)?;
    let (input, _) = symbol(')')(input)?;

    Ok((
        input,
        Statement::DoWhile(DoWhile {
            block,
            cond: Box::new(cond),
        }),
    ))
}
