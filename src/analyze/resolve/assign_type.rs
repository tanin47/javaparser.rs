use analyze::definition::{Class, CompilationUnit, Decl, Field, FieldGroup, Method, Package, Root};
use analyze::resolve::grapher::{Grapher, Node};
use analyze::resolve::scope::{EnclosingType, Scope};
use analyze::tpe::{ClassType, PackagePrefix, Prefix, Type};
use crossbeam_queue::SegQueue;
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
                    unsafe { &(*node.class) }.name.fragment
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
        fulfilled.insert(dependent_index);

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
    scope.enter_class(class);
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
    let new_type_opt = {
        let tpe = field.tpe.borrow();
        resolve_type(&tpe, scope)
    };
    match new_type_opt {
        Some(new_type) => {
            field.tpe.replace(new_type);
        }
        None => (),
    };
}

fn apply_method<'def, 'def_ref, 'scope_ref>(
    method: &'def_ref Method<'def>,
    scope: &'scope_ref mut Scope<'def, 'def_ref>,
) {
    let new_type_opt = {
        let rt = method.return_type.borrow();
        resolve_type(&rt, scope)
    };
    match new_type_opt {
        Some(new_type) => {
            method.return_type.replace(new_type);
        }
        None => (),
    };
}

fn resolve_type<'def, 'type_ref, 'def_ref, 'scope_ref>(
    tpe: &'type_ref Type<'def>,
    scope: &'scope_ref mut Scope<'def, 'def_ref>,
) -> Option<Type<'def>> {
    match tpe {
        Type::Class(class_type) => resolve_class_type(class_type, scope).map(Type::Class),
        Type::Array(array_type) => resolve_type(&array_type.elem_type, scope),
        _ => None,
    }
}

pub fn resolve_class_type<'def, 'type_ref, 'def_ref, 'scope_ref>(
    class_type: &'type_ref ClassType<'def>,
    scope: &'scope_ref Scope<'def, 'def_ref>,
) -> Option<ClassType<'def>> {
    let (result_opt, prefix_opt) = if let Some(prefix) = &class_type.prefix_opt {
        let new_prefix_opt = resolve_prefix(&prefix, scope);
        let result_opt = match &new_prefix_opt {
            Some(Prefix::Package(package)) => unsafe { (*package.def).find(class_type.name) },
            Some(Prefix::Class(class)) => match class.def_opt.get() {
                Some(def) => {
                    unsafe { (*def).find(class_type.name) }.map(|c| EnclosingType::Class(c))
                }
                None => None,
            },
            None => None,
        };
        (result_opt, new_prefix_opt)
    } else {
        (scope.resolve_type(class_type.name), None)
    };

    match result_opt {
        Some(EnclosingType::Class(class)) => Some(ClassType {
            prefix_opt: prefix_opt.map(Box::new),
            name: class_type.name,
            type_args: vec![],
            def_opt: Cell::new(Some(class)),
        }),
        Some(EnclosingType::Package(package)) => panic!(),
        None => None,
    }
}

fn resolve_prefix<'def, 'type_ref, 'def_ref, 'scope_ref>(
    prefix: &'type_ref Prefix<'def>,
    scope: &'scope_ref Scope<'def, 'def_ref>,
) -> Option<Prefix<'def>> {
    match prefix {
        Prefix::Package(package) => resolve_package_prefix(package, scope),
        Prefix::Class(class) => resolve_class_type_prefix(class, scope),
    }
}

fn resolve_package_prefix<'def, 'type_ref, 'def_ref, 'scope_ref>(
    package: &'type_ref PackagePrefix<'def>,
    scope: &'scope_ref Scope<'def, 'def_ref>,
) -> Option<Prefix<'def>> {
    match scope.resolve_package(package.name) {
        Some(p) => Some(Prefix::Package(PackagePrefix {
            name: &unsafe { &(*p) }.name,
            def: p as *const Package<'def>,
        })),
        None => None,
    }
}

fn resolve_class_type_prefix<'def, 'type_ref, 'def_ref, 'scope_ref>(
    class_type: &'type_ref ClassType<'def>,
    scope: &'scope_ref Scope<'def, 'def_ref>,
) -> Option<Prefix<'def>> {
    match scope.resolve_type(class_type.name) {
        Some(EnclosingType::Class(class)) => Some(Prefix::Class(ClassType {
            prefix_opt: None,
            name: class_type.name,
            type_args: vec![],
            def_opt: Cell::new(Some(class)),
        })),
        Some(EnclosingType::Package(package)) => Some(Prefix::Package(PackagePrefix {
            name: unsafe { &(*package).name },
            def: package,
        })),
        None => None,
    }
}

#[cfg(test)]
mod tests {
    use super::apply;
    use analyze;
    use analyze::definition::{Class, CompilationUnit, Decl, Import, Method, Package, Root};
    use analyze::resolve::merge;
    use analyze::tpe::{ClassType, PackagePrefix, Prefix, Type};
    use std::cell::{Cell, RefCell};
    use std::convert::AsRef;
    use std::ops::Deref;
    use test_common::{code, parse, span};

    #[test]
    fn test_simple() {
        let raw1 = r#"
package dev;

class Test3 extends Test2 {}
        "#
        .to_owned();
        let raw2 = r#"
package dev;

class Test {
  class Inner {}
}
        "#
        .to_owned();
        let raw3 = r#"
package dev;

class Test2 extends Test {
  Test3 method() {}
}
        "#
        .to_owned();
        let tokens1 = code(&raw1);
        let tokens2 = code(&raw2);
        let tokens3 = code(&raw3);
        let unit1 = parse(&tokens1);
        let unit2 = parse(&tokens2);
        let unit3 = parse(&tokens3);

        let root1 = analyze::build::apply(&unit1);
        let root2 = analyze::build::apply(&unit2);
        let root3 = analyze::build::apply(&unit3);
        let mut root = merge::apply(vec![root1, root2, root3]);

        apply(&mut root);

        let ret_type = root
            .find_package("dev")
            .unwrap()
            .find_class("Test2")
            .unwrap()
            .find_method("method")
            .unwrap()
            .return_type
            .borrow();

        assert_eq!(
            ret_type.deref(),
            &Type::Class(ClassType {
                prefix_opt: None,
                name: "Test3",
                type_args: vec![],
                def_opt: Cell::new(Some(
                    root.find("dev").unwrap().find_class("Test3").unwrap() as *const Class
                ))
            })
        )
    }
}
