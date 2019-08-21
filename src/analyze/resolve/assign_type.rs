use analyze::definition::{Class, CompilationUnit, Method, Package, Root};
use analyze::resolve::scope::{EnclosingType, Scope};
use analyze::tpe::{ClassType, PackagePrefix, Prefix, Type};
use std::cell::{Cell, RefCell};
use std::ops::Deref;

pub fn apply<'def, 'def_ref>(units: &'def_ref [CompilationUnit<'def>], root: &'def_ref Root<'def>) {
    for unit in units {
        let mut scope = Scope {
            root: &root,
            levels: vec![],
            specific_imports: vec![],
            wildcard_imports: vec![],
        };

        if let Some(package) = &unit.package_opt {
            for p in &package.components {
                scope.enter_package(p);
            }
        }

        for import in &unit.imports {
            scope.add_import(import)
        }

        for class in &unit.classes {
            apply_class(unsafe { &(**class) }, &mut scope);
        }
    }
}

fn apply_class<'def, 'def_ref, 'scope_ref>(
    class: &'def_ref Class<'def>,
    scope: &'scope_ref mut Scope<'def, 'def_ref>,
) {
    scope.wrap_class(class, |scope| {
        for method in &class.methods {
            apply_method(method, scope);
        }
    });
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
        Type::Class(class_type) => resolve_class_type(class_type, scope),
        Type::Array(array_type) => resolve_type(&array_type.elem_type, scope),
        _ => None,
    }
}

fn resolve_class_type<'def, 'type_ref, 'def_ref, 'scope_ref>(
    class_type: &'type_ref ClassType<'def>,
    scope: &'scope_ref Scope<'def, 'def_ref>,
) -> Option<Type<'def>> {
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
        Some(EnclosingType::Class(class)) => Some(Type::Class(ClassType {
            prefix_opt: prefix_opt.map(Box::new),
            name: class_type.name,
            type_args: vec![],
            def_opt: Cell::new(Some(class)),
        })),
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
        Some(package) => Some(Prefix::Package(PackagePrefix {
            name: unsafe { &(*package).name },
            def: package,
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
    use analyze::definition::{Class, Method, Package, Root};
    use analyze::resolve::merge;
    use analyze::tpe::{ClassType, PackagePrefix, Prefix, Type};
    use std::cell::{Cell, RefCell};
    use test_common::{code, parse, span};

    #[test]
    fn test_simple() {
        let tokens1 = code(
            r#"
package dev;

class Test {}
        "#,
        );
        let tokens2 = code(
            r#"
package dev;

class Test2 {
    dev.Test method() {}
}
        "#,
        );
        let unit1 = &parse(&tokens1);
        let unit2 = &parse(&tokens2);

        let (root1, unit1) = analyze::build::apply(&unit1);
        let (root2, unit2) = analyze::build::apply(&unit2);
        let units = vec![unit1, unit2];
        let root = merge::apply(vec![root1, root2]);

        apply(&units, &root);

        assert_eq!(
            root,
            Root {
                subpackages: vec![Package {
                    import_path: "dev".to_string(),
                    name: "dev".to_string(),
                    subpackages: vec![],
                    classes: vec![
                        Class {
                            import_path: "dev.Test".to_string(),
                            name: &span(3, 7, "Test"),
                            type_params: vec![],
                            extend_opt: None,
                            implements: vec![],
                            constructors: vec![],
                            methods: vec![],
                            field_groups: vec![],
                            classes: vec![],
                            interfaces: vec![],
                        },
                        Class {
                            import_path: "dev.Test2".to_string(),
                            name: &span(3, 7, "Test2"),
                            type_params: vec![],
                            extend_opt: None,
                            implements: vec![],
                            constructors: vec![],
                            methods: vec![Method {
                                modifiers: vec![],
                                type_params: vec![],
                                return_type: RefCell::new(Type::Class(ClassType {
                                    prefix_opt: Some(Box::new(Prefix::Package(PackagePrefix {
                                        name: "dev",
                                        def: root.find_package("dev").unwrap()
                                    }))),
                                    name: "Test",
                                    type_args: vec![],
                                    def_opt: Cell::new(Some(
                                        root.find("dev").unwrap().find_class("Test").unwrap()
                                    ))
                                })),
                                name: &span(4, 14, "method"),
                                params: vec![]
                            }],
                            field_groups: vec![],
                            classes: vec![],
                            interfaces: vec![]
                        }
                    ],
                    interfaces: vec![]
                }],
                classes: vec![],
                interfaces: vec![]
            }
        )
    }

    #[test]
    fn test_specific_import() {
        let tokens1 = code(
            r#"
package dev;

class Test {}
        "#,
        );
        let tokens2 = code(
            r#"
package dev2;

import dev.Test;

class Test2 {
    Test method() {}
}
        "#,
        );
        let unit1 = &parse(&tokens1);
        let unit2 = &parse(&tokens2);

        let (root1, unit1) = analyze::build::apply(&unit1);
        let (root2, unit2) = analyze::build::apply(&unit2);
        let units = vec![unit1, unit2];
        let root = merge::apply(vec![root1, root2]);

        apply(&units, &root);

        assert_eq!(
            root,
            Root {
                subpackages: vec![
                    Package {
                        import_path: "dev".to_string(),
                        name: "dev".to_string(),
                        subpackages: vec![],
                        classes: vec![Class {
                            import_path: "dev.Test".to_string(),
                            name: &span(3, 7, "Test"),
                            type_params: vec![],
                            extend_opt: None,
                            implements: vec![],
                            constructors: vec![],
                            methods: vec![],
                            field_groups: vec![],
                            classes: vec![],
                            interfaces: vec![],
                        },],
                        interfaces: vec![]
                    },
                    Package {
                        import_path: "dev2".to_string(),
                        name: "dev2".to_string(),
                        subpackages: vec![],
                        classes: vec![Class {
                            import_path: "dev2.Test2".to_string(),
                            name: &span(5, 7, "Test2"),
                            type_params: vec![],
                            extend_opt: None,
                            implements: vec![],
                            constructors: vec![],
                            methods: vec![Method {
                                modifiers: vec![],
                                type_params: vec![],
                                return_type: RefCell::new(Type::Class(ClassType {
                                    prefix_opt: None,
                                    name: "Test",
                                    type_args: vec![],
                                    def_opt: Cell::new(Some(
                                        root.find("dev").unwrap().find_class("Test").unwrap()
                                    ))
                                })),
                                name: &span(6, 10, "method"),
                                params: vec![]
                            }],
                            field_groups: vec![],
                            classes: vec![],
                            interfaces: vec![]
                        }],
                        interfaces: vec![]
                    }
                ],
                classes: vec![],
                interfaces: vec![]
            }
        )
    }
}
