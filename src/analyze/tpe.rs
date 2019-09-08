use analyze::definition::{Class, Package, TypeParam};
use std::cell::{Cell, Ref, RefCell};
use std::fmt::Debug;
use std::ops::Deref;
use tokenize::span::Span;

#[derive(Debug, PartialEq, Clone)]
pub enum Type<'a> {
    Primitive(PrimitiveType),
    Array(ArrayType<'a>),
    Class(ClassType<'a>),
    Parameterized(ParameterizedType<'a>),
    Void,
}

impl<'a> Type<'a> {
    pub fn to_type_arg(self) -> TypeArg<'a> {
        match self {
            Type::Array(arr) => TypeArg::Array(arr),
            Type::Class(class) => TypeArg::Class(class),
            Type::Parameterized(parameterized) => TypeArg::Parameterized(parameterized),
            Type::Void => panic!(),
            Type::Primitive(_) => panic!(),
        }
    }
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
pub struct ParameterizedType<'a> {
    pub name: &'a str,
    pub def_opt: Cell<Option<*const TypeParam<'a>>>,
}

impl<'a> ParameterizedType<'a> {
    pub fn find_inner_class(&self, name: &str) -> Option<ClassType<'a>> {
        let type_param = if let Some(type_param) = self.def_opt.get() {
            unsafe { &(*type_param) }
        } else {
            return None;
        };

        for extend in &type_param.extends {
            if let Some(inner) = extend.find_inner_class(name) {
                return Some(inner);
            }
        }

        None
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct ClassType<'a> {
    pub prefix_opt: RefCell<Option<Box<EnclosingType<'a>>>>,
    pub name: &'a str,
    pub type_args: Vec<TypeArg<'a>>,
    pub def_opt: Cell<Option<*const Class<'a>>>,
}

impl<'a> ClassType<'a> {
    pub fn get_extend_opt(&self) -> Option<ClassType<'a>> {
        let extend_class_opt = if let Some(def) = self.def_opt.get() {
            let def = unsafe { &(*def) };
            def.extend_opt.borrow()
        } else {
            return None;
        };

        let extend_class = if let Some(extend_class) = extend_class_opt.as_ref() {
            extend_class
        } else {
            return None;
        };

        Some(extend_class.clone())
        //        Some(extend_class.substitute_type_args_from(self))
    }

    // Example:
    // class Current<T> {}
    // class Subclass<A> extends Current<A> {}
    //
    // We get Current<T> where the value of T is assigned with A.
    //    pub fn substitute_type_args_from(&self, subclass: &ClassType<'a>) -> ClassType<'a> {}

    pub fn find_inner_class(&self, name: &str) -> Option<ClassType<'a>> {
        let class = if let Some(class) = self.def_opt.get() {
            unsafe { &(*class) }
        } else {
            return None;
        };

        match class.find(name) {
            Some(found) => {
                // TODO: transfer type args
                return Some(found.to_type());
            }
            None => {
                match class.extend_opt.borrow().as_ref() {
                    Some(extend) => {
                        if let Some(found) = extend.find_inner_class(name) {
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
    Parameterized(ParameterizedType<'a>),
    Array(ArrayType<'a>),
    Wildcard(WildcardType<'a>),
}

#[derive(Debug, PartialEq, Clone)]
pub struct WildcardType<'a> {
    pub name: &'a Span<'a>,
    pub super_opt: Option<Box<ReferenceType<'a>>>,
    pub extends: Vec<ReferenceType<'a>>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct PackagePrefix<'a> {
    pub prefix_opt: RefCell<Option<Box<EnclosingType<'a>>>>,
    pub name: &'a str,
    pub def: *const Package<'a>,
}

impl<'a> PackagePrefix<'a> {
    pub fn find(&self, name: &str) -> Option<EnclosingType<'a>> {
        let def = unsafe { &(*self.def) };
        def.find(name).map(|e| e.to_type())
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum EnclosingType<'a> {
    Package(PackagePrefix<'a>),
    Class(ClassType<'a>),
    Parameterized(ParameterizedType<'a>),
}

impl<'a> EnclosingType<'a> {
    pub fn get_name(&self) -> &str {
        match self {
            EnclosingType::Package(package) => package.name,
            EnclosingType::Class(class) => class.name,
            EnclosingType::Parameterized(p) => p.name,
        }
    }
    pub fn find(&self, name: &str) -> Option<EnclosingType<'a>> {
        match self {
            EnclosingType::Package(package) => package.find(name),
            EnclosingType::Class(class) => class
                .find_inner_class(name)
                .map(|c| EnclosingType::Class(c)),
            EnclosingType::Parameterized(parameterized) => parameterized
                .find_inner_class(name)
                .map(|c| EnclosingType::Class(c)),
        }
    }
    pub fn get_prefix_opt(&self) -> Option<Ref<Option<Box<EnclosingType<'a>>>>> {
        match self {
            EnclosingType::Package(package) => Some(package.prefix_opt.borrow()),
            EnclosingType::Class(class) => Some(class.prefix_opt.borrow()),
            EnclosingType::Parameterized(_) => None,
        }
    }
    pub fn set_prefix_opt(&self, prefix_opt: Option<EnclosingType<'a>>) {
        let boxed = prefix_opt.map(|e| Box::new(e));
        match self {
            EnclosingType::Package(package) => package.prefix_opt.replace(boxed),
            EnclosingType::Class(class) => class.prefix_opt.replace(boxed),
            EnclosingType::Parameterized(_) => panic!(),
        };
    }

    pub fn to_type(self) -> Type<'a> {
        match self {
            EnclosingType::Class(class) => Type::Class(class),
            EnclosingType::Parameterized(p) => Type::Parameterized(p),
            EnclosingType::Package(package) => panic!(),
        }
    }
}

unsafe impl<'a> Sync for EnclosingType<'a> {}
