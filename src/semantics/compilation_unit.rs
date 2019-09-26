use analyze::resolve::scope::Scope;
use semantics::import;
use semantics::tree::CompilationUnit;
use {analyze, parse};

pub fn apply<'def, 'def_ref>(
    unit: &parse::tree::CompilationUnit<'def>,
    scope: &mut Scope<'def, 'def_ref>,
) -> CompilationUnit<'def> {
    let mut imports = vec![];
    for im in &unit.imports {
        imports.push(import::apply(im, scope));
    }

    if let Some(package) = &unit.package_opt {
        let mut current = scope
            .root
            .find_package(package.components.first().unwrap().fragment)
            .unwrap();
        scope.enter_package(current);
        for component in &package.components[1..(package.components.len() - 1)] {
            current = current.find_package(component.fragment).unwrap();
            scope.enter_package(current);
        }
    }

    {
        let imports = analyze::build::compilation_unit::build_imports(&unit.imports);
        for im in &imports {
            scope.add_import(im);
        }
    }

    // TODO: process class

    CompilationUnit { imports }
}
