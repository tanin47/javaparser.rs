use analyze::build::{modifier, param, tpe, type_param};
use analyze::definition::Method;
use parse;
use std::cell::RefCell;

pub fn build<'a>(method: &'a parse::tree::Method<'a>) -> Method<'a> {
    let mut type_params = vec![];
    let mut params = vec![];

    for p in &method.params {
        params.push(param::build(p));
    }

    for t in &method.type_params {
        type_params.push(type_param::build(t));
    }

    Method {
        modifiers: modifier::build(&method.modifiers),
        type_params,
        return_type: RefCell::new(tpe::build(&method.return_type)),
        name: &method.name,
        params,
    }
}

//#[cfg(test)]
//mod tests {
//    use analyze::build::apply;
//    use analyze::definition::{Class, Method, Modifier, Param, Root, TypeParam};
//    use analyze::tpe::{ClassType, Prefix, PrimitiveType, Type};
//    use std::cell::{Cell, RefCell};
//    use test_common::{code, parse, span};
//
//    #[test]
//    fn test() {
//        assert_eq!(
//            apply(&parse(&code(
//                r#"
//class Test {
//    public void method(int a) {}
//    <T> boolean method2(T t) {}
//    Parent.Test method3() {}
//}
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
//                    classes: vec![],
//                    interfaces: vec![],
//                    constructors: vec![],
//                    methods: vec![
//                        Method {
//                            modifiers: vec![Modifier::Public],
//                            return_type: RefCell::new(Type::Void),
//                            name: &span(2, 17, "method"),
//                            type_params: vec![],
//                            params: vec![Param {
//                                tpe: Type::Primitive(PrimitiveType::Int),
//                                name: &span(2, 28, "a")
//                            }]
//                        },
//                        Method {
//                            modifiers: vec![],
//                            type_params: vec![TypeParam {
//                                name: &span(3, 6, "T"),
//                                extends: vec![]
//                            }],
//                            return_type: RefCell::new(Type::Primitive(PrimitiveType::Boolean)),
//                            name: &span(3, 17, "method2"),
//                            params: vec![Param {
//                                tpe: Type::Class(ClassType {
//                                    prefix_opt: None,
//                                    name: "T",
//                                    type_args: vec![],
//                                    def_opt: Cell::new(None)
//                                }),
//                                name: &span(3, 27, "t")
//                            }]
//                        },
//                        Method {
//                            modifiers: vec![],
//                            return_type: RefCell::new(Type::Class(ClassType {
//                                prefix_opt: Some(Box::new(Prefix::Class(ClassType {
//                                    prefix_opt: None,
//                                    name: "Parent",
//                                    type_args: vec![],
//                                    def_opt: Cell::new(None)
//                                }))),
//                                name: "Test",
//                                type_args: vec![],
//                                def_opt: Cell::new(None)
//                            })),
//                            name: &span(4, 17, "method3"),
//                            type_params: vec![],
//                            params: vec![]
//                        },
//                    ],
//                    field_groups: vec![],
//                    implements: vec![]
//                }],
//            }
//        )
//    }
//}
