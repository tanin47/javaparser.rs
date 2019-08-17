use tokenize::span::Span;

#[derive(Debug, PartialEq, Clone)]
pub struct Root<'a> {
    pub subpackages: Vec<Package<'a>>,
    pub classes: Vec<ClassLike<'a>>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Package<'a> {
    pub name: &'a Span<'a>,
    pub subpackages: Vec<Package<'a>>,
    pub classes: Vec<ClassLike<'a>>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum ClassLike<'a> {
    Class(Class<'a>),
}

#[derive(Debug, PartialEq, Clone)]
pub struct Class<'a> {
    pub name: &'a Span<'a>,
}
