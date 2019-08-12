use parse::combinator::{identifier, separated_nonempty_list, symbol, word};
use parse::def::annotateds;
use parse::tree::Package;
use parse::{ParseResult, Tokens};

pub fn parse(input: Tokens) -> ParseResult<Package> {
    let (input, annotateds) = annotateds::parse(input)?;

    let (input, _) = word("package")(input)?;

    let (input, components) = separated_nonempty_list(symbol('.'), identifier)(input)?;

    let (input, _) = symbol(';')(input)?;

    Ok((
        input,
        Package {
            annotateds,
            components,
        },
    ))
}
