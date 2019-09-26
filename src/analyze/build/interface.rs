use analyze::build::scope::Scope;
use analyze::build::{class, constructor, field_group, method};
use analyze::definition::{Class, Decl, Interface};
use parse;
use parse::tree::ClassBodyItem;

pub fn build<'def, 'scope_ref, 'def_ref>(
    interface: &'def_ref parse::tree::Interface<'def>,
    scope: &'scope_ref mut Scope,
) -> Interface<'def> {
    scope.wrap(interface.name.fragment, |scope| {
        let mut decls = vec![];
        let mut methods = vec![];
        let mut field_groups = vec![];

        for item in &interface.body.items {
            match item {
                ClassBodyItem::Method(m) => methods.push(method::build(m)),
                ClassBodyItem::FieldDeclarators(f) => field_groups.push(field_group::build(f)),
                ClassBodyItem::Class(c) => decls.push(Decl::Class(class::build(c, scope))),
                ClassBodyItem::Interface(i) => decls.push(Decl::Interface(build(i, scope))),
                _ => (),
            };
        }

        Interface {
            import_path: scope.get_import_path(),
            name: interface.name.clone(),
            decls,
            methods,
            field_groups,
        }
    })
}

//#[cfg(test)]
//mod tests {
//    use analyze::build::apply;
//    use analyze::definition::{
//        Class, Constructor, Field, FieldGroup, Interface, Method, Package, Root,
//    };
//    use analyze::tpe::{PrimitiveType, Type};
//    use std::cell::RefCell;
//    use test_common::{code, parse, span};
//
//    #[test]
//    fn test() {
//        assert_eq!(
//            apply(&parse(&code(
//                r#"
//interface Test {
//    void method() {}
//    int a;
//    class InnerClass {}
//}
//        "#,
//            )))
//            .0,
//            Root {
//                subpackages: vec![],
//                classes: vec![],
//                interfaces: vec![Interface {
//                    import_path: "Test".to_owned(),
//                    name: &span(1, 11, "Test"),
//                    classes: vec![Class {
//                        import_path: "Test.InnerClass".to_owned(),
//                        name: &span(4, 11, "InnerClass"),
//                        type_params: vec![],
//                        extend_opt: None,
//                        classes: vec![],
//                        interfaces: vec![],
//                        constructors: vec![],
//                        methods: vec![],
//                        field_groups: vec![],
//                        implements: vec![]
//                    }],
//                    interfaces: vec![],
//                    methods: vec![Method {
//                        modifiers: vec![],
//                        return_type: RefCell::new(Type::Void),
//                        name: &span(2, 10, "method"),
//                        type_params: vec![],
//                        params: vec![]
//                    }],
//                    field_groups: vec![FieldGroup {
//                        modifiers: vec![],
//                        items: vec![Field {
//                            tpe: Type::Primitive(PrimitiveType::Int),
//                            name: &span(3, 9, "a")
//                        },]
//                    }]
//                }],
//            }
//        )
//    }
//}
