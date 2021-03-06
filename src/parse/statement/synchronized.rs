use parse::combinator::{keyword, symbol};
use parse::id_gen::IdGen;
use parse::statement::block;
use parse::tree::{Statement, Synchronized};
use parse::{expr, ParseResult, Tokens};

pub fn parse<'def, 'r>(
    input: Tokens<'def, 'r>,
    id_gen: &mut IdGen,
) -> ParseResult<'def, 'r, Statement<'def>> {
    let (input, _) = keyword("synchronized")(input)?;
    let (input, _) = symbol('(')(input)?;
    let (input, expr) = expr::parse(input, id_gen)?;
    let (input, _) = symbol(')')(input)?;
    let (input, block) = block::parse_block(input, id_gen)?;

    Ok((
        input,
        Statement::Synchronized(Synchronized {
            expr: Box::new(expr),
            block,
        }),
    ))
}
