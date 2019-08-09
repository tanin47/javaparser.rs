use nom::IResult;
use syntax::statement::block;
use syntax::tree::{Span, Statement, WhileLoop};
use syntax::{expr, tag, word};

pub fn parse(input: Span) -> IResult<Span, Statement> {
    let (input, _) = word("while")(input)?;
    let (input, _) = tag("(")(input)?;
    let (input, cond) = expr::parse(input)?;
    let (input, _) = tag(")")(input)?;
    let (input, block) = block::parse_block_or_single_statement(input)?;

    Ok((
        input,
        Statement::WhileLoop(WhileLoop {
            cond: Box::new(cond),
            block,
        }),
    ))
}
