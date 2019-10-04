use analyze;
use analyze::definition::{Class, Package, Root};
use analyze::resolve::merge;
use parse::tree::CompilationUnit;
use parse::{apply_tokens, Tokens};
use std::cell::RefCell;
use test_common::{generate_tokens, span};
use tokenize::span::Span;
use tokenize::token::Token;

pub fn mock_class<'def, 'def_ref>(name: &'def_ref Span<'def>) -> Class<'def> {
    Class {
        import_path: name.fragment.to_owned(),
        name: name.clone(),
        type_params: vec![],
        extend_opt: RefCell::new(None),
        implements: vec![],
        constructors: vec![],
        methods: vec![],
        field_groups: vec![],
        decls: vec![],
    }
}

pub fn find_package<'r, 'def>(root: &Root<'def>, path: &str) -> &'r Package<'def> {
    let components = path.split(".").collect::<Vec<&str>>();

    let mut current = root.find(components.first().unwrap()).unwrap();

    for component in &components[1..(components.len() - 1)] {
        current = current.find(component).unwrap();
    }

    current.find_package(components.last().unwrap()).unwrap()
}

pub fn find_class<'r, 'def>(root: &Root<'def>, path: &str) -> &'r Class<'def> {
    let components = path.split(".").collect::<Vec<&str>>();
    let mut current = root.find(components.first().unwrap()).unwrap();

    for component in &components[1..(components.len() - 1)] {
        current = current.find(component).unwrap();
    }

    current.find_class(components.last().unwrap()).unwrap()
}
