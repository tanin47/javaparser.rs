use analyze::build::scope::Scope;
use analyze::build::{constructor, field_group, interface, method, type_param};
use analyze::definition::{Class, Decl};
use analyze::resolve::scope::EnclosingTypeDef;
use parse;
use parse::tree::ClassBodyItem;
use std::cell::{Cell, RefCell};

pub fn build<'def, 'scope_ref, 'def_ref>(
    class: &'def_ref parse::tree::Class<'def>,
    scope: &'scope_ref mut Scope,
) -> Class<'def> {
    scope.wrap(class.name.fragment, |scope| {
        let mut constructors = vec![];
        let mut decls = vec![];
        let mut methods = vec![];
        let mut field_groups = vec![];
        let mut type_params = vec![];
        let mut implements = vec![];

        for item in &class.body.items {
            match item {
                ClassBodyItem::Constructor(c) => constructors.push(constructor::build(c)),
                ClassBodyItem::Method(m) => methods.push(method::build(m)),
                ClassBodyItem::FieldDeclarators(f) => field_groups.push(field_group::build(f)),
                ClassBodyItem::Class(c) => decls.push(Decl::Class(build(c, scope))),
                ClassBodyItem::Interface(i) => {
                    decls.push(Decl::Interface(interface::build(i, scope)))
                }
                _ => (),
            };
        }

        for t in &class.type_params {
            type_params.push(type_param::build(t))
        }

        for i in &class.implements {
            implements.push(i.clone())
        }

        Class {
            import_path: scope.get_import_path(),
            name: class.name.clone(),
            type_params,
            extend_opt: RefCell::new(match &class.extend_opt {
                Some(extend) => Some(extend.clone()),
                None => None,
            }),
            decls,
            constructors,
            methods,
            field_groups,
            implements,
        }
    })
}

//#[cfg(test)]
//mod tests {
//    use analyze::build::apply;
//    use analyze::definition::{
//        Class, CompilationUnit, Constructor, Decl, Field, FieldGroup, Method, Package, Root,
//        TypeParam,
//    };
//    use parse::tree::{
//        ClassType, PrimitiveType, PrimitiveTypeType, ReferenceType, Type, TypeArg, Void,
//        WildcardType,
//    };
//    use std::cell::{Cell, RefCell};
//    use test_common::{generate_tokens, span};
//
//    #[test]
//    fn test() {
//        assert_eq!(
//            apply(&parse(&generate_tokens(
//                r#"
//class Test<T> extends Super<? extends T> implements Interface<T> {
//    Test() {}
//    void method() {}
//    int a;
//    class InnerClass extends Other{}
//}
//        "#,
//            ))),
//            Root {
//                subpackages: vec![],
//                units: vec![CompilationUnit {
//                    imports: vec![],
//                    main: Decl::Class(Class {
//                        import_path: "Test".to_owned(),
//                        name: span(1, 7, "Test"),
//                        type_params: vec![TypeParam {
//                            name: span(1, 12, "T"),
//                            extends: RefCell::new(vec![])
//                        }],
//                        extend_opt: RefCell::new(Some(ClassType {
//                            prefix_opt: None,
//                            name: span(1, 23, "Super"),
//                            type_args_opt: Some(vec![TypeArg::Wildcard(WildcardType {
//                                name: span(1, 29, "?"),
//                                super_opt: None,
//                                extends: vec![ReferenceType::Class(ClassType {
//                                    prefix_opt: None,
//                                    name: span(1, 39, "T"),
//                                    type_args_opt: None,
//                                    def_opt: None
//                                })]
//                            })]),
//                            def_opt: None
//                        })),
//                        decls: vec![Decl::Class(Class {
//                            import_path: "Test.InnerClass".to_owned(),
//                            name: span(5, 11, "InnerClass"),
//                            type_params: vec![],
//                            extend_opt: RefCell::new(Some(ClassType {
//                                prefix_opt: None,
//                                name: span(5, 30, "Other"),
//                                type_args_opt: None,
//                                def_opt: None
//                            })),
//                            decls: vec![],
//                            constructors: vec![],
//                            methods: vec![],
//                            field_groups: vec![],
//                            implements: vec![]
//                        })],
//                        constructors: vec![Constructor {
//                            name: span(2, 5, "Test")
//                        }],
//                        methods: vec![Method {
//                            modifiers: vec![],
//                            return_type: RefCell::new(Type::Void(Void {
//                                span: span(3, 5, "void")
//                            })),
//                            name: span(3, 10, "method"),
//                            type_params: vec![],
//                            params: vec![]
//                        }],
//                        field_groups: vec![FieldGroup {
//                            modifiers: vec![],
//                            items: vec![Field {
//                                tpe: RefCell::new(Type::Primitive(PrimitiveType {
//                                    name: span(4, 5, "int"),
//                                    tpe: PrimitiveTypeType::Int
//                                })),
//                                name: span(4, 9, "a")
//                            },]
//                        }],
//                        implements: vec![ClassType {
//                            prefix_opt: None,
//                            name: span(1, 53, "Interface"),
//                            type_args_opt: Some(vec![TypeArg::Class(ClassType {
//                                prefix_opt: None,
//                                name: span(1, 63, "T"),
//                                type_args_opt: None,
//                                def_opt: None
//                            })]),
//                            def_opt: None
//                        }]
//                    }),
//                    others: vec![]
//                }],
//            }
//        )
//    }
//}
