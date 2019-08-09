use nom::bytes::complete::{tag, take_till, take_while};
use nom::character::complete::multispace0;

use nom::multi::{many0, many_till, separated_nonempty_list};
use nom::IResult;

use nom::combinator::opt;
use syntax::comment;
use syntax::expr::atom::name;
use syntax::tree::Span;
use syntax::tree::{Import, Package};

fn parse_wildcard(input: Span) -> IResult<Span, Span> {
    let (input, _) = tag(".")(input)?;
    let (input, wildcard) = tag("*")(input)?;

    Ok((input, wildcard))
}

fn import(input: Span) -> IResult<Span, Import> {
    let (input, _) = comment::parse(input)?;
    let (input, _) = tag("import")(input)?;

    let (input, _) = comment::parse(input)?;
    let (input, static_opt) = opt(tag("static"))(input)?;

    let (input, _) = comment::parse(input)?;
    let (input, components) = separated_nonempty_list(tag("."), name::identifier)(input)?;
    let (input, wildcard_opt) = opt(parse_wildcard)(input)?;

    let (input, _) = comment::parse(input)?;
    let (input, _) = tag(";")(input)?;

    Ok((
        input,
        Import {
            is_static: static_opt.is_some(),
            components,
            is_wildcard: wildcard_opt.is_some(),
        },
    ))
}

pub fn parse(input: Span) -> IResult<Span, Vec<Import>> {
    many0(import)(input)
}
