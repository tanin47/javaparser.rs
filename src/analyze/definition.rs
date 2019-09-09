use analyze::resolve::scope::EnclosingTypeDef;
use analyze::tpe::{ClassType, EnclosingType, ParameterizedType, ReferenceType, Type};
use std::cell::{Cell, RefCell};
use tokenize::span::Span;

#[derive(Debug, PartialEq, Clone)]
pub struct Root<'a> {
    pub subpackages: Vec<Package<'a>>,
    pub units: Vec<CompilationUnit<'a>>,
}

impl<'a> Root<'a> {
    pub fn find(&self, name: &str) -> Option<EnclosingTypeDef<'a>> {
        for unit in &self.units {
            if let Some(class) = unit.find(name) {
                return Some(EnclosingTypeDef::Class(class));
            }
        }
        for package in &self.subpackages {
            if package.name.as_str() == name {
                return Some(EnclosingTypeDef::Package(package));
            }
        }

        None
    }

    pub fn find_package(&self, name: &str) -> Option<&Package<'a>> {
        match self.find(name) {
            Some(EnclosingTypeDef::Package(package)) => Some(unsafe { &(*package) }),
            Some(EnclosingTypeDef::Class(_)) => panic!(),
            None => None,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct CompilationUnit<'a> {
    pub imports: Vec<Import>,
    pub main: Decl<'a>,
    pub others: Vec<Decl<'a>>,
}
unsafe impl<'a> Send for CompilationUnit<'a> {}

impl<'a> CompilationUnit<'a> {
    pub fn find(&self, name: &str) -> Option<*const Class<'a>> {
        match &self.main {
            Decl::Class(class) => {
                if class.name.fragment == name {
                    return Some(class as *const Class<'a>);
                }
            }
            _ => (),
        }

        None
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Decl<'a> {
    Class(Class<'a>),
    Interface(Interface<'a>),
}

#[derive(Debug, PartialEq, Clone)]
pub struct Import {
    pub components: Vec<String>,
    pub is_wildcard: bool,
    pub is_static: bool,
}

#[derive(Debug, PartialEq, Clone)]
pub struct PackageDecl {
    pub components: Vec<String>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Package<'a> {
    pub import_path: String,
    pub name: String,
    pub subpackages: Vec<Package<'a>>,
    pub units: Vec<CompilationUnit<'a>>,
}

impl<'a> Package<'a> {
    pub fn find(&self, name: &str) -> Option<EnclosingTypeDef<'a>> {
        if let Some(class) = self.find_class(name) {
            return Some(EnclosingTypeDef::Class(class));
        }
        if let Some(package) = self.find_package(name) {
            return Some(EnclosingTypeDef::Package(package));
        }

        None
    }

    pub fn find_package<'b>(&self, name: &str) -> Option<&Package<'a>> {
        for package in &self.subpackages {
            if package.name.as_str() == name {
                return Some(package);
            }
        }

        None
    }

    pub fn find_class<'b>(&self, name: &str) -> Option<&Class<'a>> {
        for unit in &self.units {
            if let Some(class) = unit.find(name) {
                return Some(unsafe { &(*class) });
            }
        }

        None
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Class<'a> {
    pub import_path: String,
    pub name: &'a Span<'a>,
    // TODO: Handle class that can only be accessed within a compilation unit
    pub type_params: Vec<TypeParam<'a>>,
    pub extend_opt: RefCell<Option<ClassType<'a>>>,
    pub implements: Vec<ClassType<'a>>,
    pub constructors: Vec<Constructor<'a>>,
    pub methods: Vec<Method<'a>>,
    pub field_groups: Vec<FieldGroup<'a>>,
    pub decls: Vec<Decl<'a>>,
}
unsafe impl<'a> Sync for Class<'a> {}
unsafe impl<'a> Send for Class<'a> {}

impl<'a> Class<'a> {
    pub fn find<'b>(&self, name: &str) -> Option<&Class<'a>> {
        for decl in &self.decls {
            if let Decl::Class(class) = decl {
                if class.name.fragment == name {
                    return Some(class);
                }
            }
        }

        None
    }

    pub fn find_method(&self, name: &str) -> Option<&Method<'a>> {
        for method in &self.methods {
            if method.name.fragment == name {
                return Some(method);
            }
        }

        None
    }

    pub fn find_type_param(&self, name: &str) -> Option<&TypeParam<'a>> {
        for type_param in &self.type_params {
            if type_param.name.fragment == name {
                return Some(type_param);
            }
        }

        None
    }

    pub fn to_type(&self) -> ClassType<'a> {
        ClassType {
            prefix_opt: RefCell::new(None),
            name: self.name.fragment,
            type_args: vec![],
            def_opt: Cell::new(Some(self as *const Class<'a>)),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Interface<'a> {
    pub import_path: String,
    pub name: &'a Span<'a>,
    pub methods: Vec<Method<'a>>,
    pub field_groups: Vec<FieldGroup<'a>>,
    pub decls: Vec<Decl<'a>>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Constructor<'a> {
    pub name: &'a Span<'a>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Method<'a> {
    pub modifiers: Vec<Modifier>,
    pub type_params: Vec<TypeParam<'a>>,
    pub return_type: RefCell<Type<'a>>,
    pub name: &'a Span<'a>,
    pub params: Vec<Param<'a>>,
}
unsafe impl<'a> Sync for Method<'a> {}

#[derive(Debug, PartialEq, Clone)]
pub struct Param<'a> {
    pub tpe: Type<'a>,
    pub name: &'a Span<'a>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum TypeParamExtend<'a> {
    Class(ClassType<'a>),
    Parameterized(ParameterizedType<'a>),
}

impl<'a> TypeParamExtend<'a> {
    pub fn find_inner_class(&self, name: &str) -> Option<ClassType<'a>> {
        match self {
            TypeParamExtend::Class(class) => class.find_inner_class(name),
            TypeParamExtend::Parameterized(parameterized) => parameterized.find_inner_class(name),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct TypeParam<'a> {
    pub name: &'a Span<'a>,
    pub extends: RefCell<Vec<TypeParamExtend<'a>>>,
}

impl<'a> TypeParam<'a> {
    pub fn to_type(&self) -> ParameterizedType<'a> {
        ParameterizedType {
            name: self.name.fragment,
            def_opt: Cell::new(Some(self as *const TypeParam<'a>)),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Modifier {
    Abstract,
    Default,
    Final,
    Native,
    Private,
    Protected,
    Public,
    Static,
    Strictfp,
    Synchronized,
    Transient,
    Volatile,
}

#[derive(Debug, PartialEq, Clone)]
pub struct FieldGroup<'a> {
    pub modifiers: Vec<Modifier>,
    pub items: Vec<Field<'a>>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Field<'a> {
    pub tpe: RefCell<Type<'a>>,
    pub name: &'a Span<'a>,
}
unsafe impl<'a> Sync for Field<'a> {}
