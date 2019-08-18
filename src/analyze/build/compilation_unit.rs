use analyze::build::scope::Scope;
use analyze::build::{class, interface, package};
use analyze::referenceable::{Class, Interface, Package, Root};
use either::Either;
use parse;
use parse::tree::CompilationUnitItem;

pub fn build<'a>(unit: &'a parse::tree::CompilationUnit<'a>) -> Root<'a> {
    let mut scope = Scope { paths: vec![] };

    let (subpackages, (classes, interfaces)) = match &unit.package_opt {
        Some(package) => (
            vec![package::build(package, &unit.items, &mut scope)],
            (vec![], vec![]),
        ),
        None => (vec![], build_items(&unit.items, &mut scope)),
    };
    Root {
        subpackages,
        classes,
        interfaces,
    }
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
    use analyze::referenceable::{Class, Package, Root};
    use test_common::{code, parse, span};

    #[test]
    fn test_without_package() {
        assert_eq!(
            apply(&parse(&code(
                r#"
class Test {}
        "#,
            ))),
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
            ))),
            Root {
                subpackages: vec![Package {
                    import_path: "dev".to_owned(),
                    name: &span(1, 9, "dev"),
                    subpackages: vec![Package {
                        import_path: "dev.lilit".to_owned(),
                        name: &span(1, 13, "lilit"),
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
