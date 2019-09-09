use analyze;
use analyze::definition::{Class, Root};
use analyze::resolve::merge;
use parse::tree::CompilationUnit;
use parse::Tokens;
use std::cell::RefCell;
use test_common::{code, parse, span};
use tokenize::span::Span;
use tokenize::token::Token;

pub fn mock_class<'a>(name: &'a Span<'a>) -> Class<'a> {
    Class {
        import_path: name.fragment.to_owned(),
        name,
        type_params: vec![],
        extend_opt: RefCell::new(None),
        implements: vec![],
        constructors: vec![],
        methods: vec![],
        field_groups: vec![],
        decls: vec![],
    }
}

pub fn make_tokenss(raws: &[String]) -> Vec<Vec<Token>> {
    raws.iter()
        .map(|raw| code(raw))
        .collect::<Vec<Vec<Token>>>()
}

pub fn make_units<'r: 'unit, 'token, 'unit>(
    tokenss: &'r [Vec<Token<'token>>],
) -> Vec<CompilationUnit<'unit>> {
    tokenss
        .iter()
        .map(|tokens| parse(tokens))
        .collect::<Vec<CompilationUnit>>()
}

pub fn make_root<'r: 'root, 'unit, 'root>(units: &'r [CompilationUnit<'unit>]) -> Root<'root> {
    merge::apply(
        units
            .iter()
            .map(|unit| analyze::build::apply(unit))
            .collect::<Vec<Root>>(),
    )
}

pub fn find_class<'r, 'def>(root: &Root<'def>, path: &str) -> &'r Class<'def> {
    let components = path.split(".").collect::<Vec<&str>>();
    let mut current = root.find(components.first().unwrap()).unwrap();

    for component in &components[1..(components.len() - 1)] {
        current = current.find(component).unwrap();
    }

    current.find_class(components.last().unwrap()).unwrap()
}
