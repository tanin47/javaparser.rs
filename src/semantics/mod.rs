use analyze;
use analyze::resolve::scope::Scope;
use parse;
use semantics::tree::CompilationUnit;

pub mod compilation_unit;
pub mod import;
pub mod tree;

pub fn apply<'def>(
    target: &parse::tree::CompilationUnit<'def>,
    root: &analyze::definition::Root<'def>,
) -> CompilationUnit<'def> {
    let mut scope = Scope {
        root,
        levels: vec![],
        specific_imports: vec![],
        wildcard_imports: vec![],
    };
    compilation_unit::apply(target, &mut scope)
}
