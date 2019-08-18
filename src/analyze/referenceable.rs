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
    pub name: &'a Span<'a>,
    pub subpackages: Vec<Package<'a>>,
    pub classes: Vec<Class<'a>>,
    pub interfaces: Vec<Interface<'a>>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Class<'a> {
    pub import_path: String,
    pub name: &'a Span<'a>,
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
    pub name: &'a Span<'a>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct FieldGroup<'a> {
    pub items: Vec<Field<'a>>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Field<'a> {
    pub name: &'a Span<'a>,
}
