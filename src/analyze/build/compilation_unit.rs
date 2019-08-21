use analyze::build::scope::Scope;
use analyze::build::{class, interface, package};
use analyze::definition::{Class, CompilationUnit, Import, Interface, Package, PackageDecl, Root};
use either::Either;
use parse;
use parse::tree::CompilationUnitItem;

pub fn build<'a>(unit: &'a parse::tree::CompilationUnit<'a>) -> (Root<'a>, CompilationUnit<'a>) {
    let mut scope = Scope { paths: vec![] };

    let (subpackages, (classes, interfaces)) = match &unit.package_opt {
        Some(package) => (
            vec![package::build(package, &unit.items, &mut scope)],
            (vec![], vec![]),
        ),
        None => (vec![], build_items(&unit.items, &mut scope)),
    };

    let root = Root {
        subpackages,
        classes,
        interfaces,
    };
    let unit = CompilationUnit {
        imports: build_imports(&unit.imports),
        package_opt: match &unit.package_opt {
            Some(p) => Some(build_package(p)),
            None => None,
        },
        classes: collect_classes(&root),
    };

    (root, unit)
}

fn collect_classes<'def, 'b>(root: &'b Root<'def>) -> Vec<*const Class<'def>> {
    let mut class_pointers = vec![];
    for c in &root.classes {
        class_pointers.push(c as *const Class<'def>);
    }

    for package in &root.subpackages {
        collect_classes_from_package(package, &mut class_pointers);
    }

    class_pointers
}

fn collect_classes_from_package<'def, 'r, 'vec>(
    package: &'r Package<'def>,
    class_pointers: &'vec mut Vec<*const Class<'def>>,
) {
    for subpackage in &package.subpackages {
        collect_classes_from_package(subpackage, class_pointers);
    }

    for class in &package.classes {
        class_pointers.push(class as *const Class<'def>);
    }
}

fn build_package(package: &parse::tree::Package) -> PackageDecl {
    let mut components = vec![];

    for c in &package.components {
        components.push(c.fragment.to_owned());
    }

    PackageDecl { components }
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

pub fn build_items<'a, 'b>(
    items: &'a [CompilationUnitItem<'a>],
    scope: &'b mut Scope,
) -> (Vec<Class<'a>>, Vec<Interface<'a>>)
where
    'a: 'b,
{
    let mut classes = vec![];
    let mut interfaces = vec![];

    for item in items {
        match build_item(item, scope) {
            ChildItem::Class(class) => classes.push(class),
            ChildItem::Interface(interface) => interfaces.push(interface),
        };
    }

    (classes, interfaces)
}

enum ChildItem<'a> {
    Class(Class<'a>),
    Interface(Interface<'a>),
}

fn build_item<'a, 'b>(
    item: &'a parse::tree::CompilationUnitItem<'a>,
    scope: &'b mut Scope,
) -> ChildItem<'a>
where
    'a: 'b,
{
    match item {
        parse::tree::CompilationUnitItem::Class(c) => ChildItem::Class(class::build(c, scope)),
        parse::tree::CompilationUnitItem::Interface(i) => {
            ChildItem::Interface(interface::build(i, scope))
        }
        parse::tree::CompilationUnitItem::Annotation(annotation) => panic!(),
        parse::tree::CompilationUnitItem::Enum(enum_def) => panic!(),
    }
}

#[cfg(test)]
mod tests {
    use analyze::build::apply;
    use analyze::definition::{Class, Package, Root};
    use test_common::{code, parse, span};

    #[test]
    fn test_without_package() {
        assert_eq!(
            apply(&parse(&code(
                r#"
class Test {}
        "#,
            )))
            .0,
            Root {
                subpackages: vec![],
                interfaces: vec![],
                classes: vec![Class {
                    import_path: "Test".to_owned(),
                    name: &span(1, 7, "Test"),
                    type_params: vec![],
                    extend_opt: None,
                    implements: vec![],
                    constructors: vec![],
                    methods: vec![],
                    field_groups: vec![],
                    classes: vec![],
                    interfaces: vec![],
                }],
            }
        )
    }

    #[test]
    fn test_with_package() {
        assert_eq!(
            apply(&parse(&code(
                r#"
package dev.lilit;

class Test {}
        "#,
            )))
            .0,
            Root {
                subpackages: vec![Package {
                    import_path: "dev".to_owned(),
                    name: "dev".to_owned(),
                    subpackages: vec![Package {
                        import_path: "dev.lilit".to_owned(),
                        name: "lilit".to_owned(),
                        subpackages: vec![],
                        classes: vec![Class {
                            import_path: "dev.lilit.Test".to_owned(),
                            name: &span(3, 7, "Test"),
                            type_params: vec![],
                            extend_opt: None,
                            implements: vec![],
                            constructors: vec![],
                            methods: vec![],
                            field_groups: vec![],
                            classes: vec![],
                            interfaces: vec![]
                        }],
                        interfaces: vec![]
                    }],
                    classes: vec![],
                    interfaces: vec![]
                }],
                classes: vec![],
                interfaces: vec![]
            }
        )
    }

}
