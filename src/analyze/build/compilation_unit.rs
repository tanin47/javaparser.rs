use analyze::build::scope::Scope;
use analyze::build::{array, class, interface, package};
use analyze::definition::{Class, CompilationUnit, Decl, Interface, Package, PackageDecl, Root};
use either::Either;
use parse;

pub fn build<'def, 'r>(unit: &'r parse::tree::CompilationUnit<'def>) -> Root<'def> {
    let mut scope = Scope { paths: vec![] };

    let (subpackages, mut units) = match &unit.package_opt {
        Some(package) => (vec![package::build(package, unit, &mut scope)], vec![]),
        None => (vec![], vec![build_unit(unit, &mut scope)]),
    };

    units.push(CompilationUnit {
        imports: vec![],
        main: Decl::Class(array::apply()),
        others: vec![],
    });

    Root { subpackages, units }
}

pub fn build_imports<'def, 'r>(
    imports: &'r Vec<parse::tree::Import<'def>>,
) -> Vec<*const parse::tree::Import<'def>> {
    let mut new_imports = vec![];

    for import in imports {
        new_imports.push(import as *const parse::tree::Import)
    }

    new_imports
}

pub fn build_unit<'def, 'scope_ref, 'def_ref>(
    unit: &'def_ref parse::tree::CompilationUnit<'def>,
    scope: &'scope_ref mut Scope,
) -> CompilationUnit<'def> {
    let main = build_decl(&unit.items.first().unwrap(), scope);
    let mut others = vec![];

    for item in &unit.items[1..] {
        others.push(build_decl(item, scope));
    }

    CompilationUnit {
        imports: build_imports(&unit.imports),
        main,
        others,
    }
}

fn build_decl<'def, 'scope_ref, 'def_ref>(
    item: &'def_ref parse::tree::CompilationUnitItem<'def>,
    scope: &'scope_ref mut Scope,
) -> Decl<'def> {
    match item {
        parse::tree::CompilationUnitItem::Class(c) => Decl::Class(class::build(c, scope)),
        parse::tree::CompilationUnitItem::Interface(i) => {
            Decl::Interface(interface::build(i, scope))
        }
        parse::tree::CompilationUnitItem::Annotation(annotation) => panic!(),
        parse::tree::CompilationUnitItem::Enum(enum_def) => panic!(),
    }
}

//#[cfg(test)]
//mod tests {
//    use analyze::build::apply;
//    use analyze::definition::{Class, Package, Root};
//    use test_common::{code, parse, span};
//
//    #[test]
//    fn test_without_package() {
//        assert_eq!(
//            apply(&parse(&code(
//                r#"
//class Test {}
//        "#,
//            )))
//            .0,
//            Root {
//                subpackages: vec![],
//                interfaces: vec![],
//                classes: vec![Class {
//                    import_path: "Test".to_owned(),
//                    name: &span(1, 7, "Test"),
//                    type_params: vec![],
//                    extend_opt: None,
//                    implements: vec![],
//                    constructors: vec![],
//                    methods: vec![],
//                    field_groups: vec![],
//                    classes: vec![],
//                    interfaces: vec![],
//                }],
//            }
//        )
//    }
//
//    #[test]
//    fn test_with_package() {
//        assert_eq!(
//            apply(&parse(&code(
//                r#"
//package dev.lilit;
//
//class Test {}
//        "#,
//            )))
//            .0,
//            Root {
//                subpackages: vec![Package {
//                    import_path: "dev".to_owned(),
//                    name: "dev".to_owned(),
//                    subpackages: vec![Package {
//                        import_path: "dev.lilit".to_owned(),
//                        name: "lilit".to_owned(),
//                        subpackages: vec![],
//                        classes: vec![Class {
//                            import_path: "dev.lilit.Test".to_owned(),
//                            name: &span(3, 7, "Test"),
//                            type_params: vec![],
//                            extend_opt: None,
//                            implements: vec![],
//                            constructors: vec![],
//                            methods: vec![],
//                            field_groups: vec![],
//                            classes: vec![],
//                            interfaces: vec![]
//                        }],
//                        interfaces: vec![]
//                    }],
//                    classes: vec![],
//                    interfaces: vec![]
//                }],
//                classes: vec![],
//                interfaces: vec![]
//            }
//        )
//    }
//
//}
