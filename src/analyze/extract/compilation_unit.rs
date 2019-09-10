use analyze::definition::{CompilationUnit, Decl};
use analyze::extract::Extraction;
use analyze::resolve::scope::Scope;

pub fn apply<'def, 'def_ref>(
    unit: &CompilationUnit<'def>,
    extraction: &mut Extraction,
    scope: &mut Scope<'def, 'def_ref>,
) {
    for import in &unit.imports {
        scope.add_import(import);
    }

    apply_decl(&unit.main, extraction, scope)
}

pub fn apply_decl<'def, 'def_ref>(
    decl: &Decl<'def>,
    extraction: &mut Extraction,
    scope: &mut Scope<'def, 'def_ref>,
) {
    match decl {
        Decl::Class(class) => class::apply(class, extraction, scope),
        Decl::Interface(interface) => (),
    }
}
