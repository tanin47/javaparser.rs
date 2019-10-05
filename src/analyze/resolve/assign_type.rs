use analyze::definition::{Class, CompilationUnit, Decl, Field, FieldGroup, Method, Package, Root};
use analyze::resolve::grapher::{Grapher, Node};
use analyze::resolve::scope::{EnclosingTypeDef, Scope};
use crossbeam_queue::SegQueue;
use parse::tree::{
    ArrayType, ClassType, EnclosingType, PackagePrefix, ReferenceType, Type, TypeArg, WildcardType,
};
use std::cell::{Cell, RefCell};
use std::collections::{HashMap, HashSet};
use std::marker::PhantomData;
use std::ops::{Add, Deref};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

pub fn apply(root: &mut Root) {
    let mut grapher = Grapher::new(root);
    grapher.collect();

    let queue = SegQueue::new();

    for &node_index in grapher.pool.iter() {
        queue.push(grapher.nodes.get(node_index).unwrap());
    }

    let mut threads = vec![];

    let finished = Mutex::new(0);

    for i in 0..(num_cpus::get() - 1) {
        let builder = thread::Builder::new();
        let finished = &finished;
        let grapher = &grapher;
        let queue = &queue;
        threads.push(
            unsafe {
                builder.spawn_unchecked(move || {
                    work(i, finished, grapher, queue);
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

fn work<'def, 'def_ref, 'grapher_ref, 'queue_ref>(
    thread_index: usize,
    finished: &Mutex<usize>,
    grapher: &'grapher_ref Grapher<'def, 'def_ref>,
    queue: &'queue_ref SegQueue<&'grapher_ref Node<'def, 'def_ref>>,
) {
    loop {
        if *finished.lock().unwrap().deref() == grapher.nodes.len() {
            break;
        }
        match queue.pop() {
            Ok(node) => {
                println!(
                    "Worker {} works on {}",
                    thread_index,
                    unsafe { &(*node.class) }.name
                );
                apply_node(node, grapher, queue);
                let mut counter = finished.lock().unwrap();
                *counter += 1;
            }
            Err(_) => thread::sleep(Duration::from_millis(10)),
        };
    }
}

fn apply_node<'def, 'queue_ref, 'grapher_ref, 'def_ref>(
    node: &'grapher_ref Node<'def, 'def_ref>,
    grapher: &'grapher_ref Grapher<'def, 'def_ref>,
    queue: &'queue_ref SegQueue<&'grapher_ref Node<'def, 'def_ref>>,
) {
    apply_class(
        unsafe { &(*node.class) },
        node.scope.borrow_mut().as_mut().unwrap(),
    );

    for &dependent_index in &node.dependents {
        let dependent = grapher.nodes.get(dependent_index).unwrap();

        let mut fulfilled = dependent.fulfilled.lock().unwrap();
        fulfilled.insert(node.index);

        if fulfilled.len() == dependent.dependencies.len() {
            queue.push(dependent);
        }
    }
}

fn apply_decl<'def, 'def_ref, 'scope_ref>(
    decl: &'def_ref Decl<'def>,
    scope: &'scope_ref mut Scope<'def, 'def_ref>,
) {
    match decl {
        Decl::Class(class) => apply_class(class, scope),
        _ => (),
    };
}

fn apply_class<'def, 'def_ref, 'scope_ref>(
    class: &'def_ref Class<'def>,
    scope: &'scope_ref mut Scope<'def, 'def_ref>,
) {
    scope.enter();
    // TypeParam can be referred to in the 'extend' section. But the class itself can't.
    // So, we do double-scope here.
    for type_param in &class.type_params {
        scope.add_type_param(type_param);
    }

    let resolved_extend_opt = if let Some(extend) = class.extend_opt.borrow().as_ref() {
        match resolve_class_or_parameterized_type(extend, scope) {
            Some(Type::Class(resolved)) => Some(resolved),
            Some(_) => panic!(),
            None => None,
        }
    } else {
        None
    };

    if let Some(resolved_extend) = resolved_extend_opt {
        class.extend_opt.replace(Some(resolved_extend));
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

fn apply_field_group<'def, 'def_ref, 'scope_ref>(
    field_group: &'def_ref FieldGroup<'def>,
    scope: &'scope_ref mut Scope<'def, 'def_ref>,
) {
    for field in &field_group.items {
        apply_field(field, scope)
    }
}

fn apply_field<'def, 'def_ref, 'scope_ref>(
    field: &'def_ref Field<'def>,
    scope: &'scope_ref mut Scope<'def, 'def_ref>,
) {
    resolve_and_replace_type(&field.tpe, scope);
}

pub fn resolve_and_replace_type<'def>(cell: &RefCell<Type<'def>>, scope: &Scope<'def, '_>) {
    let new_type_opt = {
        let tpe = cell.borrow();
        resolve_type(&tpe, scope)
    };
    match new_type_opt {
        Some(new_type) => {
            cell.replace(new_type);
        }
        None => (),
    };
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

pub fn resolve_type<'def, 'type_ref, 'def_ref, 'scope_ref>(
    tpe: &'type_ref Type<'def>,
    scope: &'scope_ref Scope<'def, 'def_ref>,
) -> Option<Type<'def>> {
    match tpe {
        Type::Class(class_type) => resolve_class_or_parameterized_type(class_type, scope),
        Type::Array(array_type) => {
            resolve_array_type(array_type, scope).map(|resolved| Type::Array(resolved))
        }
        _ => None,
    }
}

pub fn resolve_array_type<'def, 'type_ref, 'def_ref, 'scope_ref>(
    array_type: &'type_ref ArrayType<'def>,
    scope: &'scope_ref Scope<'def, 'def_ref>,
) -> Option<ArrayType<'def>> {
    resolve_type(&array_type.tpe, scope).map(|elem_type| ArrayType {
        tpe: Box::new(elem_type),
        size_opt: None,
    })
}

pub fn resolve_class_or_parameterized_type<'def, 'type_ref, 'def_ref, 'scope_ref>(
    class_type: &'type_ref ClassType<'def>,
    scope: &'scope_ref Scope<'def, 'def_ref>,
) -> Option<Type<'def>> {
    resolve_enclosing_type(class_type, scope).map(|e| e.to_type())
}

pub fn resolve_type_arg<'type_ref, 'def, 'scope_ref, 'def_ref>(
    type_arg: &'type_ref TypeArg<'def>,
    scope: &'scope_ref Scope<'def, 'def_ref>,
) -> Option<TypeArg<'def>> {
    match type_arg {
        TypeArg::Class(class) => {
            resolve_class_or_parameterized_type(class, scope).map(|t| t.to_type_arg())
        }
        TypeArg::Array(array) => resolve_array_type(array, scope).map(|t| TypeArg::Array(t)),
        TypeArg::Parameterized(parameterized) => None,
        TypeArg::Wildcard(wild) => Some(TypeArg::Wildcard(resolve_wildcard_type(wild, scope))),
    }
}

pub fn resolve_reference_type<'type_ref, 'scope_ref, 'def, 'def_ref>(
    reference: &'type_ref ReferenceType<'def>,
    scope: &'scope_ref Scope<'def, 'def_ref>,
) -> ReferenceType<'def> {
    let type_opt = match reference {
        ReferenceType::Class(class) => resolve_class_or_parameterized_type(class, scope),
        ReferenceType::Array(array) => resolve_array_type(array, scope).map(|t| Type::Array(t)),
        ReferenceType::Parameterized(parameterized) => None,
    };

    type_opt
        .map(|t| t.to_reference_type())
        .unwrap_or_else(|| reference.clone())
}

pub fn resolve_wildcard_type<'type_ref, 'def, 'scope_ref, 'def_ref>(
    wildcard: &'type_ref WildcardType<'def>,
    scope: &'scope_ref Scope<'def, 'def_ref>,
) -> WildcardType<'def> {
    WildcardType {
        name: wildcard.name,
        extends: wildcard
            .extends
            .iter()
            .map(|e| resolve_reference_type(e, scope))
            .collect(),
        super_opt: match &wildcard.super_opt {
            Some(super_tpe) => Some(Box::new(resolve_reference_type(super_tpe, scope))),
            None => None,
        },
    }
}

pub fn resolve_enclosing_type<'def, 'type_ref, 'def_ref, 'scope_ref>(
    unknown_type: &'type_ref ClassType<'def>,
    scope: &'scope_ref Scope<'def, 'def_ref>,
) -> Option<EnclosingType<'def>> {
    let mut result_opt = if let Some(prefix) = unknown_type.prefix_opt.as_ref() {
        let prefix = match resolve_prefix(&prefix, scope) {
            Some(prefix) => prefix,
            None => return None,
        };

        let result = prefix
            .find(&unknown_type.name)
            .unwrap_or_else(|| EnclosingType::Class(unknown_type.clone()));

        Some(result.set_prefix_opt(Some(prefix)))
    } else {
        scope.resolve_type(&unknown_type.name)
    };

    if let Some(type_args) = &unknown_type.type_args_opt {
        if !type_args.is_empty() {
            if let Some(EnclosingType::Class(resolved)) = &mut result_opt {
                let mut resolved_type_args = vec![];
                for type_arg in type_args {
                    resolved_type_args.push(
                        resolve_type_arg(type_arg, scope).unwrap_or_else(|| type_arg.clone()),
                    );
                }
                result_opt = Some(EnclosingType::Class(ClassType {
                    prefix_opt: resolved.prefix_opt.clone(),
                    name: resolved.name.clone(),
                    type_args_opt: Some(resolved_type_args),
                    def_opt: resolved.def_opt.clone(),
                }))
            } else {
                panic!()
            }
        }
    }

    result_opt
}

fn resolve_prefix<'def, 'type_ref, 'def_ref, 'scope_ref>(
    prefix: &'type_ref EnclosingType<'def>,
    scope: &'scope_ref Scope<'def, 'def_ref>,
) -> Option<EnclosingType<'def>> {
    if let Some(ref_prefix_prefix) = prefix.get_prefix_opt() {
        if let Some(prefix_prefix) = ref_prefix_prefix {
            let prefix_prefix = resolve_prefix(prefix_prefix.deref(), scope)
                .unwrap_or_else(|| prefix_prefix.deref().clone());

            let name = prefix.get_name();

            let mut result_opt = prefix_prefix.find(prefix.get_name());

            if let Some(result) = &result_opt {
                result_opt = Some(result.set_prefix_opt(Some(prefix_prefix)))
            }

            return result_opt;
        }
    }

    match prefix {
        EnclosingType::Package(package) => resolve_package_prefix(package, scope),
        EnclosingType::Class(class) => resolve_enclosing_type(class, scope),
        EnclosingType::Parameterized(p) => Some(EnclosingType::Parameterized(p.clone())),
    }
}

fn resolve_package_prefix<'def, 'type_ref, 'def_ref, 'scope_ref>(
    package: &'type_ref PackagePrefix<'def>,
    scope: &'scope_ref Scope<'def, 'def_ref>,
) -> Option<EnclosingType<'def>> {
    match scope.resolve_package(package.name.fragment) {
        Some(p) => Some(EnclosingType::Package(PackagePrefix {
            prefix_opt: None,
            name: package.name.clone(),
            def: p as *const Package<'def>,
        })),
        None => None,
    }
}

//#[cfg(test)]
//mod tests {
//    use super::apply;
//    use analyze;
//    use analyze::definition::{Class, Package, Root, TypeParam};
//    use analyze::resolve::merge;
//    use analyze::test_common::{find_class, make_root, make_tokenss, make_units};
//    use parse::tree::{
//        ClassType, CompilationUnit, CompilationUnitItem, EnclosingType, PackagePrefix,
//        ParameterizedType, PrimitiveType, PrimitiveTypeType, ReferenceType, Type, TypeArg, Void,
//        WildcardType,
//    };
//    use std::cell::{Cell, RefCell};
//    use std::convert::AsRef;
//    use std::ops::Deref;
//    use test_common::{generate_tokens, parse, span};
//    use tokenize::token::Token;
//
//    #[test]
//    fn test_circular_parameterized() {
//        // This case proves that we need to process type params after processing the concrete types.
//        let raws = vec![
//            r#"
//package dev;
//
//class Test extends Super<Test> {
//}
//        "#
//            .to_owned(),
//            r#"
//package dev;
//
//class Super<T extends Test> {
//    class Inner {}
//    T.Inner method() {}
//}
//        "#
//            .to_owned(),
//        ];
//        let tokenss = make_tokenss(&raws);
//        let units = make_units(&tokenss);
//        let mut root = make_root(&units);
//
//        apply(&mut root);
//
//        let ret_type = find_class(&root, "dev.Super")
//            .find_method("method")
//            .unwrap()
//            .return_type
//            .borrow();
//
//        assert_eq!(
//            ret_type.deref(),
//            &Type::Class(ClassType {
//                prefix_opt: Some(Box::new(EnclosingType::Parameterized(ParameterizedType {
//                    name: span(5, 5, "T"),
//                    def: root
//                        .find_package("dev")
//                        .unwrap()
//                        .find_class("Super")
//                        .unwrap()
//                        .find_type_param("T")
//                        .unwrap() as *const TypeParam
//                }))),
//                name: span(5, 7, "Inner"),
//                type_args_opt: None,
//                def_opt: None
//            })
//        )
//    }
//
//    #[test]
//    fn test_not_allowed() {
//        // ParameterizedType CANNOT exist as a super class of a class.
//        // However, this is for code intelligence, so we allow it.
//        let raws = vec![
//            r#"
//package dev;
//
//class Test<A extends Super> {
//  class Inner extends A.SuperInner {}
//}
//        "#
//            .to_owned(),
//            r#"
//package dev;
//
//class Super {
//  class SuperInner {}
//}
//        "#
//            .to_owned(),
//        ];
//
//        let tokenss = make_tokenss(&raws);
//        let units = make_units(&tokenss);
//        let mut root = make_root(&units);
//
//        apply(&mut root);
//
//        let inner_extend_opt = find_class(&root, "dev.Test.Inner").extend_opt.borrow();
//
//        assert_eq!(
//            inner_extend_opt.as_ref().unwrap(),
//            &ClassType {
//                prefix_opt: Some(Box::new(EnclosingType::Parameterized(ParameterizedType {
//                    name: span(4, 23, "A"),
//                    def: root
//                        .find_package("dev")
//                        .unwrap()
//                        .find_class("Test")
//                        .unwrap()
//                        .find_type_param("A")
//                        .unwrap() as *const TypeParam
//                }))),
//                name: span(4, 25, "SuperInner"),
//                type_args_opt: None,
//                def_opt: None
//            }
//        )
//    }
//
//    #[test]
//    fn test_select_type_param() {
//        let raws = vec![
//            r#"
//package dev;
//
//class Test<A> extends Super {
//  class Inner extends Typed<A> {}
//}
//        "#
//            .to_owned(),
//            r#"
//package dev;
//
//class Super {
//  class A {}
//  class Typed<A> {}
//}
//        "#
//            .to_owned(),
//        ];
//
//        let tokenss = make_tokenss(&raws);
//        let units = make_units(&tokenss);
//        let mut root = make_root(&units);
//        apply(&mut root);
//
//        let inner_extend_opt = find_class(&root, "dev.Test.Inner").extend_opt.borrow();
//
//        assert_eq!(
//            inner_extend_opt.as_ref().unwrap(),
//            &ClassType {
//                prefix_opt: None,
//                name: span(4, 23, "Typed"),
//                type_args_opt: Some(vec![TypeArg::Parameterized(ParameterizedType {
//                    name: span(4, 29, "A"),
//                    def: root
//                        .find_package("dev")
//                        .unwrap()
//                        .find_class("Test")
//                        .unwrap()
//                        .find_type_param("A")
//                        .unwrap() as *const TypeParam
//                })]),
//                def_opt: Some(
//                    root.find_package("dev")
//                        .unwrap()
//                        .find_class("Super")
//                        .unwrap()
//                        .find("Typed")
//                        .unwrap() as *const Class
//                )
//            }
//        )
//    }
//
//    #[test]
//    fn test_detect_parameterized() {
//        let raws = vec![r#"
//package dev;
//
//class Test<A> {
//    A method() {}
//}
//        "#
//        .to_owned()];
//
//        let tokenss = make_tokenss(&raws);
//        let units = make_units(&tokenss);
//        let mut root = make_root(&units);
//        apply(&mut root);
//
//        let ret_type = find_class(&root, "dev.Test")
//            .find_method("method")
//            .unwrap()
//            .return_type
//            .borrow();
//
//        assert_eq!(
//            ret_type.deref(),
//            &Type::Parameterized(ParameterizedType {
//                name: span(4, 5, "A"),
//                def: root
//                    .find_package("dev")
//                    .unwrap()
//                    .find_class("Test")
//                    .unwrap()
//                    .find_type_param("A")
//                    .unwrap() as *const TypeParam
//            })
//        )
//    }
//
//    #[test]
//    fn test_resolve_from_prefix() {
//        let raws = vec![
//            r#"
//package dev;
//
//class Parent<A> {
//  class Inner {}
//}
//        "#
//            .to_owned(),
//            r#"
//package dev;
//
//class Test<T> {
//  Parent<T>.Inner method() {}
//}
//        "#
//            .to_owned(),
//        ];
//
//        let tokenss = make_tokenss(&raws);
//        let units = make_units(&tokenss);
//        let mut root = make_root(&units);
//        apply(&mut root);
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
//                prefix_opt: Some(Box::new(EnclosingType::Class(ClassType {
//                    prefix_opt: None,
//                    name: span(4, 3, "Parent"),
//                    type_args_opt: Some(vec![TypeArg::Parameterized(ParameterizedType {
//                        name: span(4, 10, "T"),
//                        def: root
//                            .find_package("dev")
//                            .unwrap()
//                            .find_class("Test")
//                            .unwrap()
//                            .find_type_param("T")
//                            .unwrap() as *const TypeParam
//                    })]),
//                    def_opt: Some(
//                        root.find_package("dev")
//                            .unwrap()
//                            .find_class("Parent")
//                            .unwrap() as *const Class
//                    )
//                }))),
//                name: span(4, 13, "Inner"),
//                type_args_opt: None,
//                def_opt: Some(
//                    root.find_package("dev")
//                        .unwrap()
//                        .find_class("Parent")
//                        .unwrap()
//                        .find("Inner")
//                        .unwrap() as *const Class
//                )
//            })
//        )
//    }
//
//    #[test]
//    fn test_inner_class_of_super_class() {
//        let raws = vec![
//            r#"
//package dev;
//
//class Test {
//  class Inner {
//    class InnerOfInner extends Super {
//      SuperInner method() {}
//    }
//  }
//}
//        "#
//            .to_owned(),
//            r#"
//package dev;
//
//class Super {
//  class SuperInner {}
//}
//        "#
//            .to_owned(),
//        ];
//        let tokenss = make_tokenss(&raws);
//        let units = make_units(&tokenss);
//        let mut root = make_root(&units);
//        apply(&mut root);
//
//        let ret_type = find_class(&root, "dev.Test.Inner.InnerOfInner")
//            .find_method("method")
//            .unwrap()
//            .return_type
//            .borrow();
//
//        assert_eq!(
//            ret_type.deref(),
//            &Type::Class(ClassType {
//                prefix_opt: None,
//                name: span(6, 7, "SuperInner"),
//                type_args_opt: None,
//                def_opt: Some(
//                    root.find_package("dev")
//                        .unwrap()
//                        .find_class("Super")
//                        .unwrap()
//                        .find("SuperInner")
//                        .unwrap() as *const Class
//                )
//            })
//        )
//    }
//
//    #[test]
//    fn test_prefix() {
//        let raws = vec![
//            r#"
//package parent.dev2;
//
//class Another {
//  class AnotherInner {}
//}
//        "#
//            .to_owned(),
//            r#"
//package dev;
//
//class Test {
//  parent.dev2.Another.AnotherInner method() {}
//}
//        "#
//            .to_owned(),
//        ];
//        let tokenss = make_tokenss(&raws);
//        let units = make_units(&tokenss);
//        let mut root = make_root(&units);
//        apply(&mut root);
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
//                prefix_opt: Some(Box::new(EnclosingType::Class(ClassType {
//                    prefix_opt: Some(Box::new(EnclosingType::Package(PackagePrefix {
//                        prefix_opt: Some(Box::new(EnclosingType::Package(PackagePrefix {
//                            prefix_opt: None,
//                            name: span(4, 3, "parent"),
//                            def: root.find_package("parent").unwrap() as *const Package
//                        }))),
//                        name: span(4, 10, "dev2"),
//                        def: root
//                            .find_package("parent")
//                            .unwrap()
//                            .find_package("dev2")
//                            .unwrap() as *const Package
//                    }))),
//                    name: span(4, 15, "Another"),
//                    type_args_opt: None,
//                    def_opt: Some(
//                        root.find("parent")
//                            .unwrap()
//                            .find("dev2")
//                            .unwrap()
//                            .find_class("Another")
//                            .unwrap() as *const Class
//                    )
//                }))),
//                name: span(4, 23, "AnotherInner"),
//                type_args_opt: None,
//                def_opt: Some(
//                    root.find("parent")
//                        .unwrap()
//                        .find("dev2")
//                        .unwrap()
//                        .find_class("Another")
//                        .unwrap()
//                        .find("AnotherInner")
//                        .unwrap() as *const Class
//                )
//            })
//        )
//    }
//
//    #[test]
//    fn test_extend() {
//        let raws = vec![
//            r#"
//package dev;
//
//class Test {}
//        "#
//            .to_owned(),
//            r#"
//package dev;
//
//class Test2 extends Test {
//  Test method() {}
//}
//        "#
//            .to_owned(),
//        ];
//        let tokenss = make_tokenss(&raws);
//        let units = make_units(&tokenss);
//        let mut root = make_root(&units);
//        apply(&mut root);
//
//        let ret_type = find_class(&root, "dev.Test2")
//            .find_method("method")
//            .unwrap()
//            .return_type
//            .borrow();
//
//        assert_eq!(
//            ret_type.deref(),
//            &Type::Class(ClassType {
//                prefix_opt: None,
//                name: span(4, 3, "Test"),
//                type_args_opt: None,
//                def_opt: Some(root.find("dev").unwrap().find_class("Test").unwrap() as *const Class)
//            })
//        )
//    }
//
//    #[test]
//    fn test_inner() {
//        let raws = vec![
//            r#"
//package dev;
//
//class Test {
//  class Inner {}
//}
//        "#
//            .to_owned(),
//            r#"
//package dev;
//
//class Test2 extends Test {}
//        "#
//            .to_owned(),
//            r#"
//package dev;
//
//class Test3 extends Test2 {
//  Inner method() {}
//}
//        "#
//            .to_owned(),
//        ];
//        let tokenss = make_tokenss(&raws);
//        let units = make_units(&tokenss);
//        let mut root = make_root(&units);
//        apply(&mut root);
//
//        let ret_type = find_class(&root, "dev.Test3")
//            .find_method("method")
//            .unwrap()
//            .return_type
//            .borrow();
//
//        assert_eq!(
//            ret_type.deref(),
//            &Type::Class(ClassType {
//                prefix_opt: None,
//                name: span(4, 3, "Inner"),
//                type_args_opt: None,
//                def_opt: Some(
//                    root.find("dev")
//                        .unwrap()
//                        .find_class("Test")
//                        .unwrap()
//                        .find("Inner")
//                        .unwrap() as *const Class
//                )
//            })
//        );
//    }
//
//    #[test]
//    fn test_method_params() {
//        let raws = vec![
//            r#"
//package dev;
//
//class Test<A extends Outer> {
//  class Inner {}
//  void method(int a, Arg<Test<A>, A, ? extends A.Inner> c) {}
//}
//        "#
//            .to_owned(),
//            r#"
//package dev;
//
//class Arg<A, B, C> {}
//        "#
//            .to_owned(),
//            r#"
//package dev;
//
//class Outer extends SuperOuter {}
//        "#
//            .to_owned(),
//            r#"
//package dev;
//
//class SuperOuter {
//    class Inner {}
//}
//        "#
//            .to_owned(),
//        ];
//        let tokenss = make_tokenss(&raws);
//        let units = make_units(&tokenss);
//        let mut root = make_root(&units);
//        apply(&mut root);
//
//        let method = find_class(&root, "dev.Test").find_method("method").unwrap();
//
//        assert_eq!(
//            method.return_type.borrow().deref(),
//            &Type::Void(Void {
//                span: span(5, 3, "void")
//            })
//        );
//        assert_eq!(
//            method.params.get(0).unwrap().tpe.borrow().deref(),
//            &Type::Primitive(PrimitiveType {
//                name: span(5, 15, "int"),
//                tpe: PrimitiveTypeType::Int
//            })
//        );
//        assert_eq!(
//            method.params.get(1).unwrap().tpe.borrow().deref(),
//            &Type::Class(ClassType {
//                prefix_opt: None,
//                name: span(5, 22, "Arg"),
//                type_args_opt: Some(vec![
//                    TypeArg::Class(ClassType {
//                        prefix_opt: None,
//                        name: span(5, 26, "Test"),
//                        type_args_opt: Some(vec![TypeArg::Parameterized(ParameterizedType {
//                            name: span(5, 31, "A"),
//                            def: find_class(&root, "dev.Test").find_type_param("A").unwrap()
//                        })]),
//                        def_opt: Some(find_class(&root, "dev.Test"))
//                    }),
//                    TypeArg::Parameterized(ParameterizedType {
//                        name: span(5, 35, "A"),
//                        def: find_class(&root, "dev.Test").find_type_param("A").unwrap()
//                    }),
//                    TypeArg::Wildcard(WildcardType {
//                        name: span(5, 38, "?"),
//                        super_opt: None,
//                        extends: vec![ReferenceType::Class(ClassType {
//                            prefix_opt: Some(Box::new(EnclosingType::Parameterized(
//                                ParameterizedType {
//                                    name: span(5, 48, "A"),
//                                    def: find_class(&root, "dev.Test")
//                                        .find_type_param("A")
//                                        .unwrap()
//                                }
//                            ))),
//                            name: span(5, 50, "Inner"),
//                            type_args_opt: None,
//                            def_opt: None
//                        })]
//                    })
//                ]),
//                def_opt: Some(find_class(&root, "dev.Arg"))
//            })
//        );
//    }
//}
