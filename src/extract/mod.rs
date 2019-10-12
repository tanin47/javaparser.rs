use analyze;
use parse::tree::{Class, CompilationUnit, Method, VariableDeclarator};
use std::any::Any;
use tokenize::span::Span;

#[cfg(test)]
pub mod test_common;

pub mod block;
pub mod compilation_unit;
pub mod def;
pub mod expr;
pub mod import;
pub mod statement;
pub mod tpe;

#[derive(Debug, PartialEq)]
pub struct Usage<'def> {
    pub span: Span<'def>,
    pub def: Definition<'def>,
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Definition<'a> {
    Package(*const analyze::definition::Package<'a>),
    Class(*const analyze::definition::Class<'a>),
    Method(*const analyze::definition::Method<'a>),
    Field(*const analyze::definition::FieldDef<'a>),
    TypeParam(*const analyze::definition::TypeParam<'a>),
    VariableDeclarator(*const VariableDeclarator<'a>),
}

impl<'a> Definition<'a> {
    pub fn ptr(&self) -> usize {
        match self {
            Definition::Package(p) => *p as usize,
            Definition::Class(c) => *c as usize,
            Definition::Method(m) => *m as usize,
            Definition::VariableDeclarator(v) => *v as usize,
            Definition::Field(f) => *f as usize,
            Definition::TypeParam(t) => *t as usize,
        }
    }
    pub fn span(&self) -> Option<&Span<'a>> {
        match self {
            Definition::Package(_) => None,
            Definition::Class(c) => unsafe { &**c }.span_opt.as_ref(),
            Definition::Method(m) => unsafe { &**m }.span_opt.as_ref(),
            Definition::Field(f) => unsafe { &**f }.span_opt.as_ref(),
            Definition::VariableDeclarator(v) => {
                let v = unsafe { &**v };
                Some(&v.name)
            }
            Definition::TypeParam(t) => unsafe { &**t }.span_opt.as_ref(),
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
