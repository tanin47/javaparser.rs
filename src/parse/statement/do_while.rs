use parse::combinator::{keyword, symbol};
use parse::statement::block;
use parse::tree::{DoWhile, Statement, WhileLoop};
use parse::{expr, ParseResult, Tokens};

pub fn parse(input: Tokens) -> ParseResult<Statement> {
    let (input, _) = keyword("do")(input)?;
    let (input, block) = block::parse_block_or_single_statement(input)?;
    let (input, _) = keyword("while")(input)?;
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
