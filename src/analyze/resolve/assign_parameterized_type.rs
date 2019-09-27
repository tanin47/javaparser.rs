use analyze::definition::{
    Class, CompilationUnit, Decl, Field, FieldGroup, Method, Package, Root, TypeParamExtend,
};
use analyze::resolve::assign_type::{
    resolve_and_replace_type, resolve_class_or_parameterized_type, resolve_type,
};
use analyze::resolve::scope::Scope;
use analyze::tpe::Type;
use crossbeam_queue::SegQueue;
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
    field: &'def_ref Field<'def>,
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

#[cfg(test)]
mod tests {
    use super::apply;
    use analyze::definition::{Class, TypeParam};
    use analyze::resolve::assign_type;
    use analyze::test_common::{find_class, make_root, make_tokenss, make_units};
    use analyze::tpe::{
        ClassType, EnclosingType, ParameterizedType, PrimitiveType, ReferenceType, Type, TypeArg,
        WildcardType,
    };
    use std::cell::{Cell, RefCell};
    use std::ops::Deref;
    use test_common::span;

    #[test]
    fn test_circular_parameterized() {
        // This case proves that we need to process type params after processing the concrete types.
        let raws = vec![
            r#"
package dev;

class Test extends Super<Test> {}
        "#
            .to_owned(),
            r#"
package dev;

class Super<T extends Test> {
    class Inner {}
    T.Inner method() {}
    T.Inner field;
}
        "#
            .to_owned(),
        ];
        let tokenss = make_tokenss(&raws);
        let units = make_units(&tokenss);
        let mut root = make_root(&units);

        assign_type::apply(&mut root);
        apply(&mut root);

        let method = find_class(&root, "dev.Super")
            .find_method("method")
            .unwrap();
        let field = find_class(&root, "dev.Super").find_field("field").unwrap();

        let expected_type = Type::Class(ClassType {
            prefix_opt: RefCell::new(Some(Box::new(EnclosingType::Parameterized(
                ParameterizedType {
                    name: "T",
                    def_opt: Cell::new(Some(
                        find_class(&root, "dev.Super").find_type_param("T").unwrap(),
                    )),
                },
            )))),
            name: "Inner",
            type_args: vec![],
            def_opt: Cell::new(Some(find_class(&root, "dev.Super.Inner"))),
        });
        assert_eq!(method.return_type.borrow().deref(), &expected_type);
        assert_eq!(field.tpe.borrow().deref(), &expected_type);
    }

    #[test]
    fn test_multi_extend() {
        // This case proves that we need to process type params after processing the concrete types.
        let raws = vec![
            r#"
package dev;

class Test<A extends Super, B extends A, C extends B> {
    C.Inner method() {}
}
        "#
            .to_owned(),
            r#"
package dev;

class Super {
    class Inner {}
}
        "#
            .to_owned(),
        ];
        let tokenss = make_tokenss(&raws);
        let units = make_units(&tokenss);
        let mut root = make_root(&units);

        assign_type::apply(&mut root);
        apply(&mut root);

        let ret_type = find_class(&root, "dev.Test")
            .find_method("method")
            .unwrap()
            .return_type
            .borrow();

        assert_eq!(
            ret_type.deref(),
            &Type::Class(ClassType {
                prefix_opt: RefCell::new(Some(Box::new(EnclosingType::Parameterized(
                    ParameterizedType {
                        name: "C",
                        def_opt: Cell::new(Some(
                            find_class(&root, "dev.Test").find_type_param("C").unwrap()
                                as *const TypeParam
                        ))
                    }
                )))),
                name: "Inner",
                type_args: vec![],
                def_opt: Cell::new(Some(find_class(&root, "dev.Super.Inner") as *const Class))
            })
        )
    }

    #[test]
    fn test_method_params() {
        let raws = vec![
            r#"
package dev;

class Test<A extends Outer> {
  class Inner {}
  void method(int a, Arg<Test<A>, A, ? extends A.Inner> c) {}
}
        "#
            .to_owned(),
            r#"
package dev;

class Arg<A, B, C> {}
        "#
            .to_owned(),
            r#"
package dev;

class Outer extends SuperOuter {}
        "#
            .to_owned(),
            r#"
package dev;

class SuperOuter {
    class Inner {}
}
        "#
            .to_owned(),
        ];
        let tokenss = make_tokenss(&raws);
        let units = make_units(&tokenss);
        let mut root = make_root(&units);
        assign_type::apply(&mut root);
        apply(&mut root);

        let method = find_class(&root, "dev.Test").find_method("method").unwrap();

        assert_eq!(method.return_type.borrow().deref(), &Type::Void);
        assert_eq!(
            method.params.get(0).unwrap().tpe.borrow().deref(),
            &Type::Primitive(PrimitiveType::Int)
        );
        assert_eq!(
            method.params.get(1).unwrap().tpe.borrow().deref(),
            &Type::Class(ClassType {
                prefix_opt: RefCell::new(None),
                name: "Arg",
                type_args: vec![
                    TypeArg::Class(ClassType {
                        prefix_opt: RefCell::new(None),
                        name: "Test",
                        type_args: vec![TypeArg::Parameterized(ParameterizedType {
                            name: "A",
                            def_opt: Cell::new(Some(
                                find_class(&root, "dev.Test").find_type_param("A").unwrap()
                            ))
                        })],
                        def_opt: Cell::new(Some(find_class(&root, "dev.Test")))
                    }),
                    TypeArg::Parameterized(ParameterizedType {
                        name: "A",
                        def_opt: Cell::new(Some(
                            find_class(&root, "dev.Test").find_type_param("A").unwrap()
                        ))
                    }),
                    TypeArg::Wildcard(WildcardType {
                        name: span(5, 38, "?"),
                        super_opt: None,
                        extends: vec![ReferenceType::Class(ClassType {
                            prefix_opt: RefCell::new(Some(Box::new(EnclosingType::Parameterized(
                                ParameterizedType {
                                    name: "A",
                                    def_opt: Cell::new(Some(
                                        find_class(&root, "dev.Test").find_type_param("A").unwrap()
                                    ))
                                }
                            )))),
                            name: "Inner",
                            type_args: vec![],
                            def_opt: Cell::new(Some(find_class(&root, "dev.SuperOuter.Inner")))
                        })]
                    })
                ],
                def_opt: Cell::new(Some(find_class(&root, "dev.Arg")))
            })
        );
    }
}
