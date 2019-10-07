use analyze;
use analyze::resolve::scope::Scope;
use parse;

pub mod block;
pub mod class;
pub mod compilation_unit;
pub mod expr;
pub mod import;
pub mod method;
pub mod statement;
//pub mod tree;

pub fn apply<'def>(
    target: &parse::tree::CompilationUnit<'def>,
    root: &analyze::definition::Root<'def>,
) {
    let mut scope = Scope {
        root,
        levels: vec![],
        specific_imports: vec![],
        wildcard_imports: vec![],
    };
    compilation_unit::apply(target, &mut scope)
}
