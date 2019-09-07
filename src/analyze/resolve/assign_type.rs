use analyze::definition::{Class, CompilationUnit, Decl, Field, FieldGroup, Method, Package, Root};
use analyze::resolve::grapher::{Grapher, Node};
use analyze::resolve::scope::{EnclosingTypeDef, Scope};
use analyze::tpe::{ClassType, EnclosingType, PackagePrefix, Type};
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
    scope.enter();
    // TODO: add type param as type args
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
        Type::Class(class_type) => resolve_class_or_parameterized_type(class_type, scope),
        Type::Array(array_type) => resolve_type(&array_type.elem_type, scope),
        _ => None,
    }
}

pub fn resolve_class_or_parameterized_type<'def, 'type_ref, 'def_ref, 'scope_ref>(
    class_type: &'type_ref ClassType<'def>,
    scope: &'scope_ref Scope<'def, 'def_ref>,
) -> Option<Type<'def>> {
    println!("{}", class_type.name);
    let result_opt = if let Some(prefix) = class_type.prefix_opt.borrow().as_ref() {
        let new_prefix_opt = resolve_prefix(&prefix, scope);
        println!("prefix {:#?}", new_prefix_opt);
        let prefix = new_prefix_opt.unwrap_or_else(|| *prefix.clone());

        let result_opt = prefix.find(class_type.name);

        if let Some(result) = &result_opt {
            result.set_prefix_opt(Some(prefix))
        }
        result_opt
    } else {
        scope.resolve_type(class_type.name)
    };

    match result_opt {
        Some(EnclosingType::Class(class)) => Some(Type::Class(class)),
        Some(EnclosingType::Parameterized(p)) => Some(Type::Parameterized(p)),
        Some(EnclosingType::Package(package)) => panic!(),
        None => None,
    }
}

fn resolve_prefix<'def, 'type_ref, 'def_ref, 'scope_ref>(
    prefix: &'type_ref EnclosingType<'def>,
    scope: &'scope_ref Scope<'def, 'def_ref>,
) -> Option<EnclosingType<'def>> {
    if let Some(prefix_prefix) = prefix.get_prefix_opt() {
        let prefix_prefix = resolve_prefix(prefix_prefix.as_ref().bo, scope)
            .unwrap_or_else(|| prefix_prefix.deref().clone());
        let name = prefix.get_name();

        let result_opt = prefix_prefix.find(prefix.get_name());

        if let Some(result) = &result_opt {
            result.set_prefix_opt(Some(prefix_prefix))
        }
        result_opt
    } else {
        match prefix {
            EnclosingType::Package(package) => resolve_package_prefix(package, scope),
            EnclosingType::Class(class) => scope.resolve_type(class.name),
            EnclosingType::Parameterized(p) => None,
        }
    }
}

fn resolve_package_prefix<'def, 'type_ref, 'def_ref, 'scope_ref>(
    package: &'type_ref PackagePrefix<'def>,
    scope: &'scope_ref Scope<'def, 'def_ref>,
) -> Option<EnclosingType<'def>> {
    match scope.resolve_package(package.name) {
        Some(p) => Some(EnclosingType::Package(PackagePrefix {
            prefix_opt: RefCell::new(None),
            name: &unsafe { &(*p) }.name,
            def: p as *const Package<'def>,
        })),
        None => None,
    }
}

#[cfg(test)]
mod tests {
    use super::apply;
    use analyze;
    use analyze::definition::{Class, Package, Root};
    use analyze::resolve::merge;
    use analyze::tpe::{ClassType, EnclosingType, PackagePrefix, Type};
    use parse::tree::{CompilationUnit, CompilationUnitItem};
    use std::cell::{Cell, RefCell};
    use std::convert::AsRef;
    use std::ops::Deref;
    use test_common::{code, parse, span};
    use tokenize::token::Token;

    #[test]
    fn test_paratermeterized_prefix() {
        // ParameterizedType can exists as a super class of a class
        let raws = vec![
            r#"
package dev;

class Test<A extends Super> {
  A.SuperInner method() {}
}
        "#
            .to_owned(),
            r#"
package dev;

class Super {
  class SuperInner {}
}
        "#
            .to_owned(),
        ];
    }

    #[test]
    fn test_not_allowed() {
        // ParameterizedType can exists as a super class of a class
        let raws = vec![
            r#"
package dev;

class Test<A extends Super> {
  class Inner extends A.SuperInner {}
}
        "#
            .to_owned(),
            r#"
package dev;

class Super {
  class SuperInner {}
}
        "#
            .to_owned(),
        ];
    }

    #[test]
    fn test_select_type_param() {
        let raws = vec![
            r#"
package dev;

class Test<A> extends Super {
  class Inner extends Typed<A> {}
}
        "#
            .to_owned(),
            r#"
package dev;

class Super {
  class A {}
  class Typed<A> {}
}
        "#
            .to_owned(),
        ];
    }

    #[test]
    fn test_resolve_from_prefix() {
        let raws = vec![
            r#"
package dev;

class Parent<A> {
  class Inner {}
}
        "#
            .to_owned(),
            r#"
package dev;

class Test<T> {
  Parent<T>.Inner method() {}
}
        "#
            .to_owned(),
        ];
    }

    #[test]
    fn test_scope_resolve_from_outer_class() {
        let raws = vec![
            r#"
package dev;

class Test<A> {
  class Inner {
    class InnerOfInner extends Super<A> {
      SuperInner method() {}
    } 
  }
}
        "#
            .to_owned(),
            r#"
package dev;

class Super<T> {
  class SuperInner {
    T method() {}
  }
}
        "#
            .to_owned(),
        ];
    }

    #[test]
    fn test_prefix() {
        let raws = vec![
            r#"
package parent.dev2;

class Another {
  class AnotherInner {}
}
        "#
            .to_owned(),
            r#"
package dev;

class Test {
  parent.dev2.Another.AnotherInner method() {}
}
        "#
            .to_owned(),
        ];
        let tokenss = raws
            .iter()
            .map(|raw| code(raw))
            .collect::<Vec<Vec<Token>>>();
        let units = tokenss
            .iter()
            .map(|tokens| parse(tokens))
            .collect::<Vec<CompilationUnit>>();

        let mut root = merge::apply(
            units
                .iter()
                .map(|unit| analyze::build::apply(unit))
                .collect::<Vec<Root>>(),
        );

        apply(&mut root);

        let ret_type = root
            .find_package("dev")
            .unwrap()
            .find_class("Test")
            .unwrap()
            .find_method("method")
            .unwrap()
            .return_type
            .borrow();

        assert_eq!(
            ret_type.deref(),
            &Type::Class(ClassType {
                prefix_opt: RefCell::new(Some(Box::new(EnclosingType::Class(ClassType {
                    prefix_opt: RefCell::new(Some(Box::new(EnclosingType::Package(
                        PackagePrefix {
                            prefix_opt: RefCell::new(Some(Box::new(EnclosingType::Package(
                                PackagePrefix {
                                    prefix_opt: RefCell::new(None),
                                    name: "parent",
                                    def: root.find_package("parent").unwrap() as *const Package
                                }
                            )))),
                            name: "dev2",
                            def: root
                                .find_package("parent")
                                .unwrap()
                                .find_package("dev2")
                                .unwrap() as *const Package
                        }
                    )))),
                    name: "Another",
                    type_args: vec![],
                    def_opt: Cell::new(Some(
                        root.find("parent")
                            .unwrap()
                            .find("dev2")
                            .unwrap()
                            .find_class("Another")
                            .unwrap() as *const Class
                    ))
                })))),
                name: "AnotherInner",
                type_args: vec![],
                def_opt: Cell::new(Some(
                    root.find("parent")
                        .unwrap()
                        .find("dev2")
                        .unwrap()
                        .find_class("Another")
                        .unwrap()
                        .find("AnotherInner")
                        .unwrap() as *const Class
                ))
            })
        )
    }

    #[test]
    fn test_extend() {
        let raws = vec![
            r#"
package dev;

class Test {}
        "#
            .to_owned(),
            r#"
package dev;

class Test2 extends Test {
  Test method() {}
}
        "#
            .to_owned(),
        ];
        let tokenss = raws
            .iter()
            .map(|raw| code(raw))
            .collect::<Vec<Vec<Token>>>();
        let units = tokenss
            .iter()
            .map(|tokens| parse(tokens))
            .collect::<Vec<CompilationUnit>>();

        let mut root = merge::apply(
            units
                .iter()
                .map(|unit| analyze::build::apply(unit))
                .collect::<Vec<Root>>(),
        );

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
                prefix_opt: RefCell::new(None),
                name: "Test",
                type_args: vec![],
                def_opt: Cell::new(Some(
                    root.find("dev").unwrap().find_class("Test").unwrap() as *const Class
                ))
            })
        )
    }

    #[test]
    fn test_inner() {
        let raws = vec![
            r#"
package dev;

class Test {
  class Inner {}
}
        "#
            .to_owned(),
            r#"
package dev;

class Test2 extends Test {}
        "#
            .to_owned(),
            r#"
package dev;

class Test3 extends Test2 {
  Inner method() {}
}
        "#
            .to_owned(),
        ];
        let tokenss = raws
            .iter()
            .map(|raw| code(raw))
            .collect::<Vec<Vec<Token>>>();
        let units = tokenss
            .iter()
            .map(|tokens| parse(tokens))
            .collect::<Vec<CompilationUnit>>();

        let mut root = merge::apply(
            units
                .iter()
                .map(|unit| analyze::build::apply(unit))
                .collect::<Vec<Root>>(),
        );

        apply(&mut root);

        let ret_type = root
            .find_package("dev")
            .unwrap()
            .find_class("Test3")
            .unwrap()
            .find_method("method")
            .unwrap()
            .return_type
            .borrow();

        assert_eq!(
            ret_type.deref(),
            &Type::Class(ClassType {
                prefix_opt: RefCell::new(None),
                name: "Inner",
                type_args: vec![],
                def_opt: Cell::new(Some(
                    root.find("dev")
                        .unwrap()
                        .find_class("Test")
                        .unwrap()
                        .find("Inner")
                        .unwrap() as *const Class
                ))
            })
        );
    }
}
