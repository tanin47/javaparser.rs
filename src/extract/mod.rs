use analyze::definition::{Class, Method, Package};
use parse::tree::{CompilationUnit, VariableDeclarator};
use tokenize::span::Span;

pub mod block;
pub mod class;
pub mod compilation_unit;
pub mod import;
pub mod method;
pub mod package;

#[derive(Debug, PartialEq)]
pub struct Usage<'def> {
    pub span: Span<'def>,
    pub def: Definition<'def>,
}

#[derive(Debug, PartialEq)]
pub enum Definition<'a> {
    Package(*const Package<'a>),
    Class(*const Class<'a>),
    Method(*const Method<'a>),
    VariableDeclarator(*const VariableDeclarator<'a>),
}

#[derive(Debug, PartialEq)]
pub struct Overlay<'def> {
    pub defs: Vec<Definition<'def>>,
    pub usages: Vec<Usage<'def>>,
}

pub fn apply<'def, 'def_ref>(unit: &'def_ref CompilationUnit<'def>) -> Overlay<'def> {
    let mut overlay = Overlay {
        defs: vec![],
        usages: vec![],
    };

    compilation_unit::apply(unit, &mut overlay);

    overlay
}
