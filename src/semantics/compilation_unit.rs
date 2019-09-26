use analyze::resolve::scope::Scope;
use semantics::import;
use semantics::tree::CompilationUnit;
use {analyze, parse};

pub fn apply<'def, 'def_ref>(
    target: &parse::tree::CompilationUnit<'def>,
    scope: &mut Scope<'def, 'def_ref>,
) -> CompilationUnit<'def> {
    let mut imports = vec![];

    for im in &target.imports {
        imports.push(import::apply(im, scope));
    }

    CompilationUnit { imports }
}
