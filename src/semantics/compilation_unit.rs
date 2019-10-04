use analyze::resolve::scope::Scope;
use parse::tree::CompilationUnitItem;
use semantics::{class, import};
use {analyze, parse};

pub fn apply<'def, 'def_ref, 'scope_ref>(
    unit: &'def_ref parse::tree::CompilationUnit<'def>,
    scope: &'scope_ref mut Scope<'def, 'def_ref>,
) {
    if let Some(package) = &unit.package_opt {
        enter_package(package, scope);
    } else {
        scope.enter();
    }

    for im in &unit.imports {
        scope.add_import(im);
        import::apply(im, scope);
    }

    for item in &unit.items {
        apply_item(item, scope);
    }

    if let Some(package) = &unit.package_opt {
        leave_package(package, scope);
    } else {
        scope.leave();
    }
}

fn apply_item<'def, 'def_ref, 'scope_ref>(
    item: &'def_ref CompilationUnitItem<'def>,
    scope: &'scope_ref mut Scope<'def, 'def_ref>,
) {
    match item {
        CompilationUnitItem::Class(c) => class::apply(c, scope),
        CompilationUnitItem::Interface(_) => panic!(),
        CompilationUnitItem::Annotation(_) => panic!(),
        CompilationUnitItem::Enum(_) => panic!(),
    };
}

fn enter_package<'def, 'def_ref, 'scope_ref>(
    package: &'def_ref parse::tree::Package<'def>,
    scope: &'scope_ref mut Scope<'def, 'def_ref>,
) {
    if let Some(prefix) = &package.prefix_opt {
        enter_package(prefix, scope);
        scope
            .levels
            .last()
            .unwrap()
            .enclosing_opt
            .as_ref()
            .unwrap()
            .find_package(package.name.fragment);
    } else {
        scope.enter_package(scope.root.find_package(package.name.fragment).unwrap());
    }
}

fn leave_package<'def, 'def_ref, 'scope_ref>(
    package: &'def_ref parse::tree::Package<'def>,
    scope: &'scope_ref mut Scope<'def, 'def_ref>,
) {
    if let Some(prefix) = &package.prefix_opt {
        enter_package(prefix, scope);
        scope.leave();
    } else {
        scope.leave();
    }
}
