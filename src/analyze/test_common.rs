use analyze;
use analyze::definition::{Class, Package, Root};
use analyze::resolve::merge;
use parse::tree::CompilationUnit;
use parse::{apply_tokens, Tokens};
use std::cell::RefCell;
use test_common::{generate_tokens, span};
use tokenize::span::Span;
use tokenize::token::Token;

pub fn find_package<'r, 'def>(root: &'r Root<'def>, path: &str) -> &'r Package<'def> {
    let components = path.split(".").collect::<Vec<&str>>();

    if components.len() == 1 {
        return root.find_package(components.first().unwrap()).unwrap();
    }

    let mut current = root.find(components.first().unwrap()).unwrap();

    for component in &components[1..(components.len() - 1)] {
        current = current.find(component).unwrap();
    }

    current.find_package(components.last().unwrap()).unwrap()
}

pub fn find_class<'r, 'def>(root: &'r Root<'def>, path: &str) -> &'r Class<'def> {
    let components = path.split(".").collect::<Vec<&str>>();

    if components.len() == 1 {
        return root.find_class(components.first().unwrap()).unwrap();
    }

    let mut current = root.find(components.first().unwrap()).unwrap();

    for component in &components[1..(components.len() - 1)] {
        current = current.find(component).unwrap();
    }

    current.find_class(components.last().unwrap()).unwrap()
}
