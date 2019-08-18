use analyze::tpe::{ClassType, ReferenceType, Type};
use tokenize::span::Span;

#[derive(Debug, PartialEq, Clone)]
pub struct Root<'a> {
    pub subpackages: Vec<Package<'a>>,
    pub classes: Vec<Class<'a>>,
    pub interfaces: Vec<Interface<'a>>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Package<'a> {
    pub import_path: String,
    pub name: String,
    pub subpackages: Vec<Package<'a>>,
    pub classes: Vec<Class<'a>>,
    pub interfaces: Vec<Interface<'a>>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Class<'a> {
    pub import_path: String,
    pub name: &'a Span<'a>,
    // TODO: Handle class that can only be accessed within a compilation unit
    pub type_params: Vec<TypeParam<'a>>,
    pub extend_opt: Option<ClassType<'a>>,
    pub implements: Vec<ClassType<'a>>,
    pub constructors: Vec<Constructor<'a>>,
    pub methods: Vec<Method<'a>>,
    pub field_groups: Vec<FieldGroup<'a>>,
    pub classes: Vec<Class<'a>>,
    pub interfaces: Vec<Interface<'a>>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Interface<'a> {
    pub import_path: String,
    pub name: &'a Span<'a>,
    pub methods: Vec<Method<'a>>,
    pub field_groups: Vec<FieldGroup<'a>>,
    pub classes: Vec<Class<'a>>,
    pub interfaces: Vec<Interface<'a>>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Constructor<'a> {
    pub name: &'a Span<'a>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Method<'a> {
    pub modifiers: Vec<Modifier>,
    pub type_params: Vec<TypeParam<'a>>,
    pub return_type: Type<'a>,
    pub name: &'a Span<'a>,
    pub params: Vec<Param<'a>>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Param<'a> {
    pub tpe: Type<'a>,
    pub name: &'a Span<'a>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct TypeParam<'a> {
    pub name: &'a Span<'a>,
    pub extends: Vec<ClassType<'a>>,
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
    pub tpe: Type<'a>,
    pub name: &'a Span<'a>,
}
