use analyze::referenceable::Class;
use std::cell::{Cell, RefCell};
use std::fmt::Debug;
use tokenize::span::Span;

#[derive(Debug, PartialEq, Clone)]
pub enum Type<'a> {
    Primitive(PrimitiveType),
    Array(ArrayType<'a>),
    Class(ClassType<'a>),
    Void,
}

#[derive(Debug, PartialEq, Clone)]
pub enum PrimitiveType {
    Boolean,
    Byte,
    Char,
    Double,
    Float,
    Int,
    Long,
    Short,
}

#[derive(Debug, PartialEq, Clone)]
pub struct ArrayType<'a> {
    pub elem_type: Box<Type<'a>>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct ClassType<'a> {
    pub prefix_opt: Option<Box<Prefix<'a>>>,
    pub name: &'a str,
    pub def_opt: Cell<Option<*const Class<'a>>>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct PackagePrefix<'a> {
    pub name: &'a str,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Prefix<'a> {
    Package(PackagePrefix<'a>),
    Class(ClassType<'a>),
}
