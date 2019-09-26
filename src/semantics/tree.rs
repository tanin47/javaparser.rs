use tokenize::span::Span;
use {analyze, parse};

#[derive(Debug, PartialEq, Clone)]
pub struct CompilationUnit<'a> {
    //    pub package: Package<'a>,
    pub imports: Vec<Import<'a>>,
    //    pub main: Decl<'a>,
    //    pub others: Vec<Decl<'a>>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Package<'a> {
    pub prefix_opt: Option<Box<Package<'a>>>,
    pub span: Span<'a>,
    pub def_opt: Option<*const analyze::definition::Package<'a>>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Import<'a> {
    pub span: Span<'a>,
    pub prefix_opt: Option<ImportPrefix<'a>>,
    pub is_static: bool,
    pub is_wildcard: bool,
    pub def_opt: Option<ImportDef<'a>>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct ImportPrefix<'a> {
    pub span: Span<'a>,
    pub prefix_opt: Option<Box<ImportPrefix<'a>>>,
    pub def_opt: Option<ImportPrefixDef<'a>>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum ImportPrefixDef<'a> {
    Package(*const analyze::definition::Package<'a>),
    Class(*const analyze::definition::Class<'a>),
}

#[derive(Debug, PartialEq, Clone)]
pub enum ImportDef<'a> {
    Package(*const analyze::definition::Package<'a>),
    Class(*const analyze::definition::Class<'a>),
}
