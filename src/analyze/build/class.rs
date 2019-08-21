use analyze::build::scope::Scope;
use analyze::build::{constructor, field_group, interface, method, tpe, type_param};
use analyze::definition::Class;
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
        let mut type_params = vec![];
        let mut implements = vec![];

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

        for t in &class.type_params {
            type_params.push(type_param::build(t))
        }

        for i in &class.implements {
            implements.push(tpe::build_class(i))
        }

        Class {
            import_path: scope.get_import_path(),
            name: &class.name,
            type_params,
            extend_opt: match &class.extend_opt {
                Some(extend) => Some(tpe::build_class(extend)),
                None => None,
            },
            classes,
            interfaces,
            constructors,
            methods,
            field_groups,
            implements,
        }
    })
}

#[cfg(test)]
mod tests {
    use analyze::build::apply;
    use analyze::definition::{
        Class, Constructor, Field, FieldGroup, Method, Package, Root, TypeParam,
    };
    use analyze::tpe::{ClassType, PrimitiveType, ReferenceType, Type, TypeArg, WildcardType};
    use std::cell::{Cell, RefCell};
    use test_common::{code, parse, span};

    #[test]
    fn test() {
        assert_eq!(
            apply(&parse(&code(
                r#"
class Test<T> extends Super<? extends T> implements Interface<T> {
    Test() {}
    void method() {}
    int a;
    class InnerClass {}
}
        "#,
            )))
            .0,
            Root {
                subpackages: vec![],
                interfaces: vec![],
                classes: vec![Class {
                    import_path: "Test".to_owned(),
                    name: &span(1, 7, "Test"),
                    type_params: vec![TypeParam {
                        name: &span(1, 12, "T"),
                        extends: vec![]
                    }],
                    extend_opt: Some(ClassType {
                        prefix_opt: None,
                        name: "Super",
                        type_args: vec![TypeArg::Wildcard(WildcardType {
                            name: &span(1, 29, "?"),
                            super_opt: None,
                            extends: vec![ReferenceType::Class(ClassType {
                                prefix_opt: None,
                                name: "T",
                                type_args: vec![],
                                def_opt: Cell::new(None)
                            })]
                        })],
                        def_opt: Cell::new(None)
                    }),
                    classes: vec![Class {
                        import_path: "Test.InnerClass".to_owned(),
                        name: &span(5, 11, "InnerClass"),
                        type_params: vec![],
                        extend_opt: None,
                        classes: vec![],
                        interfaces: vec![],
                        constructors: vec![],
                        methods: vec![],
                        field_groups: vec![],
                        implements: vec![]
                    }],
                    interfaces: vec![],
                    constructors: vec![Constructor {
                        name: &span(2, 5, "Test")
                    }],
                    methods: vec![Method {
                        modifiers: vec![],
                        return_type: RefCell::new(Type::Void),
                        name: &span(3, 10, "method"),
                        type_params: vec![],
                        params: vec![]
                    }],
                    field_groups: vec![FieldGroup {
                        modifiers: vec![],
                        items: vec![Field {
                            tpe: Type::Primitive(PrimitiveType::Int),
                            name: &span(4, 9, "a")
                        },]
                    }],
                    implements: vec![ClassType {
                        prefix_opt: None,
                        name: "Interface",
                        type_args: vec![TypeArg::Class(ClassType {
                            prefix_opt: None,
                            name: "T",
                            type_args: vec![],
                            def_opt: Cell::new(None)
                        })],
                        def_opt: Cell::new(None)
                    }]
                }],
            }
        )
    }
}
