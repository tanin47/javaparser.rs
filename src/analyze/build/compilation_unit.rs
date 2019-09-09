use analyze::build::scope::Scope;
use analyze::build::{class, interface, package};
use analyze::definition::{
    Class, CompilationUnit, Decl, Import, Interface, Package, PackageDecl, Root,
};
use either::Either;
use parse;
use parse::tree::CompilationUnitItem;

pub fn build<'a>(unit: &'a parse::tree::CompilationUnit<'a>) -> Root<'a> {
    let mut scope = Scope { paths: vec![] };

    let (subpackages, units) = match &unit.package_opt {
        Some(package) => (vec![package::build(package, unit, &mut scope)], vec![]),
        None => (vec![], vec![build_unit(unit, &mut scope)]),
    };

    Root { subpackages, units }
}

fn build_imports(imports: &Vec<parse::tree::Import>) -> Vec<Import> {
    let mut new_imports = vec![];

    for import in imports {
        let mut components = vec![];

        for c in &import.components {
            components.push(c.fragment.to_owned())
        }

        new_imports.push(Import {
            components,
            is_wildcard: import.is_wildcard,
            is_static: import.is_static,
        })
    }

    new_imports
}

pub fn build_unit<'a, 'b>(
    unit: &'a parse::tree::CompilationUnit<'a>,
    scope: &'b mut Scope,
) -> CompilationUnit<'a>
where
    'a: 'b,
{
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

fn build_decl<'a, 'b>(
    item: &'a parse::tree::CompilationUnitItem<'a>,
    scope: &'b mut Scope,
) -> Decl<'a>
where
    'a: 'b,
{
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
