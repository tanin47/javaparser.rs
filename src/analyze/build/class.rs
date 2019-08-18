use analyze::build::scope::Scope;
use analyze::build::{constructor, field_group, interface, method};
use analyze::referenceable::Class;
use parse;
use parse::tree::ClassBodyItem;

pub fn build<'a, 'b>(class: &'a parse::tree::Class<'a>, scope: &'b mut Scope) -> Class<'a>
where
    'a: 'b,
{
    scope.wrap(class.name.fragment, |scope| {
        let mut constructors = vec![];
        let mut classes = vec![];
        let mut interfaces = vec![];
        let mut methods = vec![];
        let mut field_groups = vec![];

        for item in &class.body.items {
            match item {
                ClassBodyItem::Constructor(c) => constructors.push(constructor::build(c)),
                ClassBodyItem::Method(m) => methods.push(method::build(m)),
                ClassBodyItem::FieldDeclarators(f) => field_groups.push(field_group::build(f)),
                ClassBodyItem::Class(c) => classes.push(build(c, scope)),
                ClassBodyItem::Interface(i) => interfaces.push(interface::build(i, scope)),
                _ => (),
            };
        }

        Class {
            import_path: scope.get_import_path(),
            name: &class.name,
            classes,
            interfaces,
            constructors,
            methods,
            field_groups,
        }
    })
}

#[cfg(test)]
mod tests {
    use analyze::build::apply;
    use analyze::referenceable::{Class, Constructor, Field, FieldGroup, Method, Package, Root};
    use analyze::tpe::Type;
    use test_common::{code, parse, span};

    #[test]
    fn test() {
        assert_eq!(
            apply(&parse(&code(
                r#"
class Test {
    Test() {}
    void method() {}
    int a, b;
    class InnerClass {}
}
        "#,
            ))),
            Root {
                subpackages: vec![],
                interfaces: vec![],
                classes: vec![Class {
                    import_path: "Test".to_owned(),
                    name: &span(1, 7, "Test"),
                    classes: vec![Class {
                        import_path: "Test.InnerClass".to_owned(),
                        name: &span(5, 11, "InnerClass"),
                        classes: vec![],
                        interfaces: vec![],
                        constructors: vec![],
                        methods: vec![],
                        field_groups: vec![]
                    }],
                    interfaces: vec![],
                    constructors: vec![Constructor {
                        name: &span(2, 5, "Test")
                    }],
                    methods: vec![Method {
                        modifiers: vec![],
                        return_type: Type::Void,
                        name: &span(3, 10, "method"),
                        params: vec![]
                    }],
                    field_groups: vec![FieldGroup {
                        items: vec![
                            Field {
                                name: &span(4, 9, "a")
                            },
                            Field {
                                name: &span(4, 12, "b")
                            },
                        ]
                    }]
                }],
            }
        )
    }
}
