use nom::IResult;
use syntax::statement::block;
use syntax::tree::{Span, Statement, Synchronized};
use syntax::{expr, tag, word};

pub fn parse(input: Span) -> IResult<Span, Statement> {
    let (input, _) = word("synchronized")(input)?;
    let (input, _) = tag("(")(input)?;
    let (input, expr) = expr::parse(input)?;
    let (input, _) = tag(")")(input)?;
    let (input, block) = block::parse_block(input)?;

    Ok((
        input,
        Statement::Synchronized(Synchronized {
            expr: Box::new(expr),
            block,
        }),
    ))
}
