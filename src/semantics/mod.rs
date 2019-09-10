use analyze;
use parse;

pub mod compilation_unit;
pub mod tree;

pub fn apply<'def>(
    target: &parse::tree::CompilationUnit<'def>,
    index: &analyze::definition::Root<'def>,
) -> CompilationUnit {
    compilation_unit::apply(target, index)
}
