use parse::combinator::{keyword, symbol};
use parse::statement::block;
use parse::tree::{Statement, WhileLoop};
use parse::{expr, ParseResult, Tokens};

pub fn parse(input: Tokens) -> ParseResult<Statement> {
    let (input, _) = keyword("while")(input)?;
    let (input, _) = symbol('(')(input)?;
    let (input, cond) = expr::parse(input)?;
    let (input, _) = symbol(')')(input)?;
    let (input, block) = block::parse_block_or_single_statement(input)?;

    Ok((
        input,
        Statement::WhileLoop(WhileLoop {
            cond: Box::new(cond),
            block,
        }),
    ))
}
