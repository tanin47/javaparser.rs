use analyze::definition::{
    Class, CompilationUnit, Decl, FieldDef, FieldGroup, Method, Package, Root, TypeParamExtend,
};
use analyze::resolve::assign_type::{
    resolve_and_replace_type, resolve_class_or_parameterized_type, resolve_type,
};
use analyze::resolve::scope::Scope;
use crossbeam_queue::SegQueue;
use parse::tree::Type;
use std::sync::Mutex;
use std::thread;

#[derive(Debug)]
pub struct Node<'def, 'def_ref> {
    pub unit: *const CompilationUnit<'def>,
    pub scope: Scope<'def, 'def_ref>,
}
unsafe impl<'def, 'def_ref> Send for Node<'def, 'def_ref> {}

pub fn apply(root: &mut Root) {
    let queue = SegQueue::new();
    let mut scope = Scope {
        root,
        levels: vec![],
        specific_imports: vec![],
        wildcard_imports: vec![],
    };
    collect_root(&queue, &mut scope, root);

    let mut threads = vec![];

    for i in 0..(num_cpus::get() - 1) {
        let builder = thread::Builder::new();
        let queue = &queue;
        threads.push(
            unsafe {
                builder.spawn_unchecked(move || {
                    work(i, queue);
                })
            }
            .unwrap(),
        )
    }

    for thread in threads {
        match thread.join() {
            Ok(_) => (),
            Err(_) => panic!(),
        }
    }
}

fn collect_root<'def, 'def_ref, 'scope, 'queue>(
    queue: &'queue SegQueue<Node<'def, 'def_ref>>,
    scope: &'scope mut Scope<'def, 'def_ref>,
    root: &'def_ref Root<'def>,
) {
    for unit in &root.units {
        queue.push(Node {
            unit: unit as *const CompilationUnit<'def>,
            scope: scope.clone(),
        });
    }
    for subpackage in &root.subpackages {
        collect_package(queue, scope, subpackage);
    }
}

fn collect_package<'def, 'def_ref, 'scope, 'queue>(
    queue: &'queue SegQueue<Node<'def, 'def_ref>>,
    scope: &'scope mut Scope<'def, 'def_ref>,
    package: &'def_ref Package<'def>,
) {
    scope.enter_package(package);
    for subpackage in &package.subpackages {
        collect_package(queue, scope, subpackage);
    }
    for unit in &package.units {
        queue.push(Node {
            unit: unit as *const CompilationUnit<'def>,
            scope: scope.clone(),
        });
    }
    scope.leave();
}

fn work(thread_index: usize, queue: &SegQueue<Node>) {
    loop {
        match queue.pop() {
            Ok(node) => apply_node(node),
            Err(_) => break,
        };
    }
}

fn apply_node(mut node: Node) {
    apply_unit(unsafe { &(*node.unit) }, &mut node.scope)
}

fn apply_unit<'def, 'def_ref>(
    unit: &'def_ref CompilationUnit<'def>,
    scope: &mut Scope<'def, 'def_ref>,
) {
    for import in &unit.imports {
        scope.add_import(unsafe { &**import });
    }

    apply_decl(&unit.main, scope);

    for decl in &unit.others {
        apply_decl(decl, scope);
    }
}

fn apply_decl<'def, 'def_ref>(decl: &'def_ref Decl<'def>, scope: &mut Scope<'def, 'def_ref>) {
    match decl {
        Decl::Class(class) => apply_class(class, scope),
        Decl::Interface(interface) => (),
    }
}

fn apply_class<'def, 'def_ref>(class: &'def_ref Class<'def>, scope: &mut Scope<'def, 'def_ref>) {
    scope.enter();
    for type_param in &class.type_params {
        scope.add_type_param(type_param);

        let mut new_extends = vec![];
        for (index, extend) in type_param.extends.borrow().iter().enumerate() {
            if let Some(resolved) = resolve_type_param_extend(extend, scope) {
                new_extends.push(resolved);
            } else {
                new_extends.push(extend.clone());
            }
        }

        type_param.extends.replace(new_extends);
    }

    scope.enter_class(class);
    // TypeParam should be recognized before traversing into the super classes.
    for type_param in &class.type_params {
        scope.add_type_param(type_param);
    }
    for method in &class.methods {
        apply_method(method, scope);
    }
    for field_group in &class.field_groups {
        apply_field_group(field_group, scope);
    }
    for inner_decl in &class.decls {
        apply_decl(inner_decl, scope);
    }

    scope.leave();
    scope.leave();
}

fn apply_method<'def, 'def_ref, 'scope_ref>(
    method: &'def_ref Method<'def>,
    scope: &'scope_ref mut Scope<'def, 'def_ref>,
) {
    resolve_and_replace_type(&method.return_type, scope);

    for param in &method.params {
        resolve_and_replace_type(&param.tpe, scope);
    }
}

fn apply_field_group<'def, 'def_ref, 'scope_ref>(
    field_group: &'def_ref FieldGroup<'def>,
    scope: &'scope_ref mut Scope<'def, 'def_ref>,
) {
    for field in &field_group.items {
        apply_field(field, scope)
    }
}

fn apply_field<'def, 'def_ref, 'scope_ref>(
    field: &'def_ref FieldDef<'def>,
    scope: &'scope_ref mut Scope<'def, 'def_ref>,
) {
    resolve_and_replace_type(&field.tpe, scope);
}

fn resolve_type_param_extend<'def, 'def_ref>(
    origin: &'def_ref TypeParamExtend<'def>,
    scope: &Scope<'def, 'def_ref>,
) -> Option<TypeParamExtend<'def>> {
    let tpe = match origin {
        TypeParamExtend::Class(class) => resolve_class_or_parameterized_type(class, scope),
        TypeParamExtend::Parameterized(parameterized) => {
            Some(Type::Parameterized(parameterized.clone()))
        }
    };

    match tpe {
        Some(Type::Class(class)) => Some(TypeParamExtend::Class(class)),
        Some(Type::Parameterized(parameterized)) => {
            Some(TypeParamExtend::Parameterized(parameterized))
        }
        None => None,
        _ => panic!(),
    }
}

//#[cfg(test)]
//mod tests {
//    use super::apply;
//    use analyze::definition::{Class, TypeParam};
//    use analyze::resolve::assign_type;
//    use analyze::test_common::find_class;
//    use parse::tree::{
//        ClassType, EnclosingType, ParameterizedType, PrimitiveType, PrimitiveTypeType,
//        ReferenceType, Type, TypeArg, Void, WildcardType,
//    };
//    use std::cell::{Cell, RefCell};
//    use std::ops::Deref;
//    use test_common::{span, span2};
//
//    #[test]
//    fn test_circular_parameterized() {
//        // This case proves that we need to process type params after processing the concrete types.
//        let (files, root) = apply_assign_parameterized_type!(
//            r#"
//package dev;
//
//class Test extends Super<Test> {}
//        "#,
//            r#"
//package dev;
//
//class Super<T extends Test> {
//    class Inner {}
//    T.Inner method() {}
//    T.Inner field;
//}
//        "#
//        );
//
//        let method = find_class(&root, "dev.Super")
//            .find_method("method")
//            .unwrap();
//        let field = find_class(&root, "dev.Super").find_field("field").unwrap();
//
//        assert_eq!(
//            method.return_type.borrow().deref(),
//            &Type::Class(ClassType {
//                prefix_opt: Some(Box::new(EnclosingType::Parameterized(ParameterizedType {
//                    name: span2(5, 5, "T", files.get(1).unwrap().deref()),
//                    def: find_class(&root, "dev.Super").find_type_param("T").unwrap(),
//                },))),
//                name: span2(5, 7, "Inner", files.get(1).unwrap().deref()),
//                type_args_opt: None,
//                def_opt: Some(find_class(&root, "dev.Super.Inner")),
//            })
//        );
//        assert_eq!(
//            field.tpe.borrow().deref(),
//            &Type::Class(ClassType {
//                prefix_opt: Some(Box::new(EnclosingType::Parameterized(ParameterizedType {
//                    name: span2(6, 5, "T", files.get(1).unwrap().deref()),
//                    def: find_class(&root, "dev.Super").find_type_param("T").unwrap(),
//                },))),
//                name: span2(6, 7, "Inner", files.get(1).unwrap().deref()),
//                type_args_opt: None,
//                def_opt: Some(find_class(&root, "dev.Super.Inner")),
//            })
//        );
//    }
//
//    #[test]
//    fn test_multi_extend() {
//        // This case proves that we need to process type params after processing the concrete types.
//        let (files, root) = apply_assign_parameterized_type!(
//            r#"
//package dev;
//
//class Test<A extends Super, B extends A, C extends B> {
//    C.Inner method() {}
//}
//        "#,
//            r#"
//package dev;
//
//class Super {
//    class Inner {}
//}
//        "#
//        );
//
//        let ret_type = find_class(&root, "dev.Test")
//            .find_method("method")
//            .unwrap()
//            .return_type
//            .borrow();
//
//        assert_eq!(
//            ret_type.deref(),
//            &Type::Class(ClassType {
//                prefix_opt: Some(Box::new(EnclosingType::Parameterized(ParameterizedType {
//                    name: span2(4, 5, "C", files.get(0).unwrap().deref()),
//                    def: find_class(&root, "dev.Test").find_type_param("C").unwrap()
//                        as *const TypeParam
//                }))),
//                name: span2(4, 7, "Inner", files.get(0).unwrap().deref()),
//                type_args_opt: None,
//                def_opt: Some(find_class(&root, "dev.Super.Inner") as *const Class)
//            })
//        )
//    }
//
//    #[test]
//    fn test_method_params() {
//        let (files, root) = apply_assign_parameterized_type!(
//            r#"
//package dev;
//
//class Test<A extends Outer> {
//  class Inner {}
//  void method(int a, Arg<Test<A>, A, ? extends A.Inner> c) {}
//}
//        "#,
//            r#"
//package dev;
//
//class Arg<A, B, C> {}
//        "#,
//            r#"
//package dev;
//
//class Outer extends SuperOuter {}
//        "#,
//            r#"
//package dev;
//
//class SuperOuter {
//    class Inner {}
//}
//        "#
//        );
//
//        let method = find_class(&root, "dev.Test").find_method("method").unwrap();
//        let method_file = files.get(0).unwrap().deref();
//
//        assert_eq!(
//            method.return_type.borrow().deref(),
//            &Type::Void(Void {
//                span: span2(5, 3, "void", method_file)
//            })
//        );
//        assert_eq!(
//            method.params.get(0).unwrap().tpe.borrow().deref(),
//            &Type::Primitive(PrimitiveType {
//                name: span2(5, 15, "int", method_file),
//                tpe: PrimitiveTypeType::Int
//            })
//        );
//        assert_eq!(
//            method.params.get(1).unwrap().tpe.borrow().deref(),
//            &Type::Class(ClassType {
//                prefix_opt: None,
//                name: span2(5, 22, "Arg", method_file),
//                type_args_opt: Some(vec![
//                    TypeArg::Class(ClassType {
//                        prefix_opt: None,
//                        name: span2(5, 26, "Test", method_file),
//                        type_args_opt: Some(vec![TypeArg::Parameterized(ParameterizedType {
//                            name: span2(5, 31, "A", method_file),
//                            def: find_class(&root, "dev.Test").find_type_param("A").unwrap()
//                        })]),
//                        def_opt: Some(find_class(&root, "dev.Test"))
//                    }),
//                    TypeArg::Parameterized(ParameterizedType {
//                        name: span2(5, 35, "A", method_file),
//                        def: find_class(&root, "dev.Test").find_type_param("A").unwrap()
//                    }),
//                    TypeArg::Wildcard(WildcardType {
//                        name: span2(5, 38, "?", method_file),
//                        super_opt: None,
//                        extends: vec![ReferenceType::Class(ClassType {
//                            prefix_opt: Some(Box::new(EnclosingType::Parameterized(
//                                ParameterizedType {
//                                    name: span2(5, 48, "A", method_file),
//                                    def: find_class(&root, "dev.Test")
//                                        .find_type_param("A")
//                                        .unwrap()
//                                }
//                            ))),
//                            name: span2(5, 50, "Inner", method_file),
//                            type_args_opt: None,
//                            def_opt: Some(find_class(&root, "dev.SuperOuter.Inner"))
//                        })]
//                    })
//                ]),
//                def_opt: Some(find_class(&root, "dev.Arg"))
//            })
//        );
//    }
//}
