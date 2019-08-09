use nom::bytes::complete::{tag, take_till, take_while};
use nom::character::complete::multispace0;

use nom::multi::{many_till, separated_nonempty_list};
use nom::IResult;

use syntax::comment;
use syntax::def::annotateds;
use syntax::expr::atom::name;
use syntax::tree::Package;
use syntax::tree::Span;

pub fn parse(input: Span) -> IResult<Span, Package> {
    let (input, annotateds) = annotateds::parse(input)?;

    let (input, _) = comment::parse(input)?;
    let (input, _) = tag("package")(input)?;

    let (input, _) = comment::parse(input)?;
    let (input, components) = separated_nonempty_list(tag("."), name::identifier)(input)?;

    let (input, _) = tag(";")(input)?;

    Ok((
        input,
        Package {
            annotateds,
            components,
        },
    ))
}
