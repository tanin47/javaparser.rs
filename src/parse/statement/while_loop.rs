use parse::combinator::{keyword, symbol};
use parse::id_gen::IdGen;
use parse::statement::block;
use parse::tree::{Statement, WhileLoop};
use parse::{expr, ParseResult, Tokens};

pub fn parse<'def, 'r>(
    input: Tokens<'def, 'r>,
    id_gen: &mut IdGen,
) -> ParseResult<'def, 'r, Statement<'def>> {
    let (input, _) = keyword("while")(input)?;
    let (input, _) = symbol('(')(input)?;
    let (input, cond) = expr::parse(input, id_gen)?;
    let (input, _) = symbol(')')(input)?;
    let (input, block) = block::parse_block_or_single_statement(input, id_gen)?;

    Ok((
        input,
        Statement::WhileLoop(WhileLoop {
            cond: Box::new(cond),
            block,
        }),
    ))
}
