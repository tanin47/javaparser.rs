use analyze::definition::Package;
use analyze::extract::{compilation_unit, Extraction};
use analyze::resolve::scope::Scope;

pub fn apply<'def, 'def_ref>(
    package: &Package<'def>,
    extraction: &mut Extraction,
    scope: &mut Scope<'def, 'def_ref>,
) {
    scope.enter_package(package);

    for subpackage in &package.subpackages {
        apply(subpackage, extraction, scope);
    }

    for unit in &package.units {
        compilation_unit::apply(unit, extraction, scope);
    }

    scope.leave();
}
