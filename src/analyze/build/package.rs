use analyze::build::scope::Scope;
use analyze::build::{class, compilation_unit};
use analyze::definition::{Class, Package};
use parse::tree::CompilationUnitItem;
use tokenize::span::Span;
use {analyze, parse};

pub fn build<'def, 'scope_ref, 'def_ref>(
    package: &'def_ref parse::tree::Package<'def>,
    unit: &'def_ref parse::tree::CompilationUnit<'def>,
    scope: &'scope_ref mut Scope,
) -> Package<'def> {
    let prefix_opt = match &package.prefix_opt {
        Some(prefix) => Some(build_prefix(&prefix, scope)),
        None => None,
    };

    scope.push(package.name.fragment);

    let current = Package {
        import_path: scope.get_import_path(),
        name: package.name.fragment.to_owned(),
        subpackages: vec![],
        units: vec![compilation_unit::build_unit(unit, scope)],
    };

    let result = match prefix_opt {
        Some(mut prefix) => {
            prefix.subpackages.push(current);
            prefix
        }
        None => current,
    };

    pop_scope(&result, scope);

    result
}

fn pop_scope(package: &analyze::definition::Package, scope: &mut Scope) {
    scope.pop();

    assert!(package.subpackages.len() <= 1);

    for subpackage in &package.subpackages {
        pop_scope(subpackage, scope);
    }
}

fn build_prefix<'def, 'scope_ref, 'def_ref>(
    package: &'def_ref parse::tree::Package<'def>,
    scope: &'scope_ref mut Scope,
) -> Package<'def> {
    let prefix_opt = match &package.prefix_opt {
        Some(prefix) => Some(build_prefix(&prefix, scope)),
        None => None,
    };

    scope.push(package.name.fragment);

    let current = Package {
        import_path: scope.get_import_path(),
        name: package.name.fragment.to_owned(),
        subpackages: vec![],
        units: vec![],
    };

    match prefix_opt {
        Some(mut prefix) => {
            prefix.subpackages.push(current);
            prefix
        }
        None => current,
    }
}
//fn build_nested<'def, 'scope_ref, 'def_ref, 'com_ref>(
//    components: &'com_ref [Span],
//    unit: &'def_ref parse::tree::CompilationUnit<'def>,
//    scope: &'scope_ref mut Scope,
//) -> Package<'def> {
//    scope.wrap(components[0].fragment, |scope| {
//        if components.len() == 1 {
//            let unit = compilation_unit::build_unit(unit, scope);
//            Package {
//                import_path: scope.get_import_path(),
//                name: components[0].fragment.to_owned(),
//                subpackages: vec![],
//                units: vec![unit],
//            }
//        } else {
//            Package {
//                import_path: scope.get_import_path(),
//                name: components[0].fragment.to_owned(),
//                subpackages: vec![build_nested(&components[1..], unit, scope)],
//                units: vec![],
//            }
//        }
//    })
//}
