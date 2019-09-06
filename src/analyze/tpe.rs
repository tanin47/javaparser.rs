use analyze::definition::{Class, Package};
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
pub enum ReferenceType<'a> {
    Array(ArrayType<'a>),
    Class(ClassType<'a>),
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
    pub prefix_opt: Option<Box<EnclosingType<'a>>>,
    pub name: &'a str,
    pub type_args: Vec<TypeArg<'a>>,
    pub def_opt: Cell<Option<*const Class<'a>>>,
}

impl<'a> ClassType<'a> {
    pub fn find_class(&self, name: &str) -> Option<ClassType<'a>> {
        let class = if let Some(class) = self.def_opt.get() {
            unsafe { &(*class) }
        } else {
            return None;
        };

        match class.find(name) {
            Some(found) => {
                let found = unsafe { &(*found) };
                // TODO: transfer type args
                return Some(found.to_type());
            }
            None => {
                match class.extend_opt.borrow().as_ref() {
                    Some(extend) => {
                        if let Some(found) = extend.find_class(name) {
                            // TODO: transfer type args
                            return Some(found);
                        }
                    }
                    None => (),
                }
            }
        };

        None
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum TypeArg<'a> {
    Class(ClassType<'a>),
    Array(ArrayType<'a>),
    Wildcard(WildcardType<'a>),
}

#[derive(Debug, PartialEq, Clone)]
pub struct WildcardType<'a> {
    pub name: &'a Span<'a>,
    pub super_opt: Option<Box<ReferenceType<'a>>>,
    pub extends: Vec<ReferenceType<'a>>,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct PackagePrefix<'a> {
    pub name: &'a str,
    pub def: *const Package<'a>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum EnclosingType<'a> {
    Package(PackagePrefix<'a>),
    Class(ClassType<'a>),
}

unsafe impl<'a> Sync for EnclosingType<'a> {}
