use analyze::definition::{Class, Method, Package};
use parse::tree::{CompilationUnit, VariableDeclarator};
use std::any::Any;
use tokenize::span::Span;

#[cfg(test)]
#[macro_use]
pub mod test_common;

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

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Definition<'a> {
    Package(*const Package<'a>),
    Class(*const Class<'a>),
    Method(*const Method<'a>),
    VariableDeclarator(*const VariableDeclarator<'a>),
}

impl<'a> Definition<'a> {
    pub fn ptr(&self) -> usize {
        match self {
            Definition::Package(p) => *p as usize,
            Definition::Class(c) => *c as usize,
            Definition::Method(m) => *m as usize,
            Definition::VariableDeclarator(v) => *v as usize,
        }
    }
    pub fn span(&self) -> Option<&Span<'a>> {
        match self {
            Definition::Package(_) => None,
            Definition::Class(c) => {
                let c = unsafe { &**c };
                Some(&c.name)
            }
            Definition::Method(m) => {
                let m = unsafe { &**m };
                Some(&m.name)
            }
            Definition::VariableDeclarator(v) => {
                let v = unsafe { &**v };
                Some(&v.name)
            }
        }
    }
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
