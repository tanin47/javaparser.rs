use analyze::resolve::scope::EnclosingTypeDef;
use parse;
use parse::tree::{ClassType, InvocationContext, ParameterizedType, Type, VariableDeclarator};
use std::cell::{Cell, RefCell};
use std::collections::HashSet;
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
    pub imports: Vec<*const parse::tree::Import<'a>>,
    pub main: Decl<'a>,
    pub others: Vec<Decl<'a>>,
}
unsafe impl<'a> Send for CompilationUnit<'a> {}

impl<'a> CompilationUnit<'a> {
    pub fn find(&self, name: &str) -> Option<*const Class<'a>> {
        match &self.main {
            Decl::Class(class) => {
                if class.name == name {
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
pub struct Class<'def> {
    pub name: &'def str,
    pub parse_opt: Option<*const parse::tree::Class<'def>>,
    // TODO: Handle class that can only be accessed within a compilation unit
    pub type_params: Vec<TypeParam<'def>>,
    pub extend_opt: RefCell<Option<ClassType<'def>>>,
    pub implements: Vec<ClassType<'def>>,
    pub constructors: Vec<Constructor<'def>>,
    pub methods: Vec<Method<'def>>,
    pub field_groups: Vec<FieldGroup<'def>>,
    pub decls: Vec<Decl<'def>>,
}
unsafe impl<'a> Sync for Class<'a> {}
unsafe impl<'a> Send for Class<'a> {}

impl<'a> Class<'a> {
    pub fn find<'b>(&self, name: &str) -> Option<&Class<'a>> {
        for decl in &self.decls {
            if let Decl::Class(class) = decl {
                if class.name == name {
                    return Some(class);
                }
            }
        }

        None
    }

    pub fn find_field(&self, name: &str) -> Option<&FieldDef<'a>> {
        for group in &self.field_groups {
            for field in &group.items {
                if &field.name == name {
                    return Some(field);
                }
            }
        }

        None
    }

    pub fn find_method(&self, name: &str) -> Option<&Method<'a>> {
        for method in &self.methods {
            if &method.name == name {
                return Some(method);
            }
        }

        None
    }

    pub fn find_type_param(&self, name: &str) -> Option<&TypeParam<'a>> {
        for type_param in &self.type_params {
            if &type_param.name == name {
                return Some(type_param);
            }
        }

        None
    }

    pub fn to_type(&self) -> ClassType<'a> {
        ClassType {
            prefix_opt: None,
            name: self.name.to_owned(),
            span_opt: None,
            type_args_opt: None,
            def_opt: Some(self as *const Class<'a>),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Interface<'a> {
    pub import_path: String,
    pub name: Span<'a>,
    pub methods: Vec<Method<'a>>,
    pub field_groups: Vec<FieldGroup<'a>>,
    pub decls: Vec<Decl<'a>>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Constructor<'a> {
    pub name: Span<'a>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Method<'a> {
    pub modifiers: HashSet<Modifier>,
    pub type_params: Vec<TypeParam<'a>>,
    pub return_type: RefCell<Type<'a>>,
    pub name: String,
    pub params: Vec<Param<'a>>,
    pub parse_opt: Option<*const parse::tree::Method<'a>>,
}
unsafe impl<'a> Sync for Method<'a> {}

#[derive(Debug, PartialEq, Clone)]
pub struct Param<'a> {
    pub tpe: RefCell<Type<'a>>,
    pub name: Span<'a>,
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

    pub fn find_field(&self, name: &str, context: &InvocationContext) -> Option<Field<'a>> {
        match self {
            TypeParamExtend::Class(class) => class.find_field(name, context),
            TypeParamExtend::Parameterized(parameterized) => {
                parameterized.find_field(name, context)
            }
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct TypeParam<'a> {
    pub name: String,
    pub extends: RefCell<Vec<TypeParamExtend<'a>>>,
    pub parse_opt: Option<*const parse::tree::TypeParam<'a>>,
}

impl<'a> TypeParam<'a> {
    // This method makes no sense. We can't use type param's name as the type. That is wrong
    //    pub fn to_type(&self) -> ParameterizedType<'a> {
    //        ParameterizedType {
    //            name: self.name,
    //            def_opt: self as *const TypeParam<'a>,
    //        }
    //    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
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
    pub modifiers: HashSet<Modifier>,
    pub items: Vec<FieldDef<'a>>,
    pub parse_opt: Option<*const parse::tree::FieldDeclarators<'a>>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct FieldDef<'a> {
    pub tpe: RefCell<Type<'a>>,
    pub name: String,
    pub parse_opt: Option<*const parse::tree::VariableDeclarator<'a>>,
}
unsafe impl<'a> Sync for FieldDef<'a> {}

#[derive(Debug, PartialEq, Clone)]
pub struct Field<'a> {
    pub tpe: Type<'a>,
    pub def: *const FieldDef<'a>,
}
