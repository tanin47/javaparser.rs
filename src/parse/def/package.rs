use parse::combinator::{identifier, keyword, separated_nonempty_list, symbol};
use parse::def::annotateds;
use parse::id_gen::IdGen;
use parse::tree::Package;
use parse::{ParseResult, Tokens};

pub fn parse<'def, 'r>(
    input: Tokens<'def, 'r>,
    id_gen: &mut IdGen,
) -> ParseResult<'def, 'r, Package<'def>> {
    let (input, annotateds) = annotateds::parse(input, id_gen)?;

    let (input, _) = keyword("package")(input)?;

    let (input, components) = separated_nonempty_list(symbol('.'), identifier)(input)?;

    let (input, _) = symbol(';')(input)?;

    let mut prefix_opt: Option<Package> = None;

    for component in &components[0..(components.len() - 1)] {
        prefix_opt = Some(Package {
            prefix_opt: prefix_opt.map(Box::new),
            annotateds: vec![],
            name: component.clone(),
            def_opt: None,
        });
    }

    Ok((
        input,
        Package {
            prefix_opt: prefix_opt.map(Box::new),
            annotateds,
            name: components.last().unwrap().clone(),
            def_opt: None,
        },
    ))
}
