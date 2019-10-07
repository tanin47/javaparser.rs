use analyze::resolve::scope::Scope;
use parse::tree::{
    EnclosingType, Expr, FieldAccess, FieldAccessPrefix, InvocationContext, PackagePrefix,
    ParameterizedType, ResolvedName, StaticClass, StaticType,
};
use semantics::expr;
use std::ops::Deref;

pub fn apply<'def, 'def_ref, 'scope_ref>(
    field_access: &'def_ref FieldAccess<'def>,
    scope: &'scope_ref mut Scope<'def, 'def_ref>,
) {
    apply_field_access(field_access, scope);
}

pub fn apply_field_access<'def, 'def_ref, 'scope_ref>(
    field_access: &'def_ref FieldAccess<'def>,
    scope: &'scope_ref mut Scope<'def, 'def_ref>,
) -> Option<FieldAccessPrefix<'def>> {
    let new_prefix_opt = apply_prefix(
        unsafe { field_access.prefix.try_borrow_unguarded() }.unwrap(),
        scope,
    );

    if let Some(new_prefix) = new_prefix_opt {
        field_access.prefix.replace(Box::new(new_prefix));
    }

    match field_access.prefix.borrow().as_ref() {
        FieldAccessPrefix::Package(p) => {
            if let Some(tpe) = p.find(&field_access.name) {
                if let EnclosingType::Package(p) = tpe {
                    return Some(FieldAccessPrefix::Package(p));
                } else {
                    field_access.tpe_opt.replace(Some(tpe.to_type()));
                }
            }
        }
        FieldAccessPrefix::Expr(Expr::StaticClass(static_class)) => {
            if let Some(field) = static_class.tpe.find_field(
                field_access.name.fragment,
                &InvocationContext { only_static: true },
            ) {
                field_access
                    .tpe_opt
                    .replace(Some(field.tpe.borrow().deref().clone()));
            }
        }
        FieldAccessPrefix::Expr(e) => {
            if let Some(tpe) = e.tpe_opt() {
                if let Some(field) = tpe.find_field(
                    field_access.name.fragment,
                    &InvocationContext { only_static: false },
                ) {
                    field_access
                        .tpe_opt
                        .replace(Some(field.tpe.borrow().deref().clone()));
                }
            }
        }
    }

    None
}

fn apply_prefix<'def, 'def_ref, 'scope_ref>(
    prefix: &'def_ref FieldAccessPrefix<'def>,
    scope: &'scope_ref mut Scope<'def, 'def_ref>,
) -> Option<FieldAccessPrefix<'def>> {
    let ex = match prefix {
        FieldAccessPrefix::Expr(e) => e,
        // The below only happens when we process the node more than once.
        FieldAccessPrefix::Package(p) => return None,
    };

    println!("E {:#?}", ex);

    match ex {
        Expr::FieldAccess(f) => {
            if let Some(new_prefix) = apply_field_access(f, scope) {
                return Some(new_prefix);
            }
        }
        Expr::Name(n) => {
            if let Some(resolved) = scope.resolve_name(&n.name) {
                println!("Resolve {:#?}", resolved);
                match resolved {
                    ResolvedName::Package(p) => {
                        return Some(FieldAccessPrefix::Package(PackagePrefix {
                            prefix_opt: None,
                            name: n.name.clone(),
                            def: p,
                        }))
                    }
                    ResolvedName::Class(c) => {
                        return Some(FieldAccessPrefix::Expr(Expr::StaticClass(StaticClass {
                            tpe: StaticType::Class(unsafe { &*c }.to_type(&n.name)),
                        })))
                    }
                    ResolvedName::TypeParam(p) => {
                        return Some(FieldAccessPrefix::Expr(Expr::StaticClass(StaticClass {
                            tpe: StaticType::Parameterized(ParameterizedType {
                                name: n.name.clone(),
                                def: p,
                            }),
                        })))
                    }
                    other => n.resolved_opt.set(Some(other)),
                };
            }
        }
        other => expr::apply(other, scope),
    };

    None
}

#[cfg(test)]
mod tests {
    use analyze::test_common::find_class;
    use parse::tree::{
        ClassBodyItem, CompilationUnitItem, Expr, PrimitiveType, PrimitiveTypeType, Statement,
        Type, TypeParam,
    };
    use std::ops::Deref;
    use test_common::span2;
    use {analyze, semantics};

    #[test]
    fn test_both_instance_and_static() {
        let (files, root) = apply_semantics!(
            r#"
package dev;

class Test<T> {
  T a;
  static boolean c;
  void method() {
    Test<Another> s; 
    int b = s.a.num;
    int d = s.c;
  }
}
        "#,
            r#"
package dev;

class Another {
  int num;
}
        "#
        );

        let class = unwrap!(
            CompilationUnitItem::Class,
            &files.first().unwrap().unit.items.get(0).unwrap()
        );
        let method = unwrap!(ClassBodyItem::Method, &class.body.items.get(2).unwrap());
        {
            let var = unwrap!(
                Statement::VariableDeclarators,
                &method.block_opt.as_ref().unwrap().stmts.get(1).unwrap()
            );

            let field_access = unwrap!(
                Expr::FieldAccess,
                var.declarators.first().unwrap().expr_opt.as_ref().unwrap()
            );
            let tpe = unwrap!(
                Type::Primitive,
                field_access.tpe_opt.borrow().clone().unwrap()
            );
            assert_eq!(
                tpe,
                PrimitiveType {
                    name: span2(4, 3, "int", files.get(1).unwrap().deref()),
                    tpe: PrimitiveTypeType::Int
                }
            );
        }
        {
            let var = unwrap!(
                Statement::VariableDeclarators,
                &method.block_opt.as_ref().unwrap().stmts.get(2).unwrap()
            );
            let field_access = unwrap!(
                Expr::FieldAccess,
                var.declarators.first().unwrap().expr_opt.as_ref().unwrap()
            );

            let tpe = unwrap!(
                Type::Primitive,
                field_access.tpe_opt.borrow().clone().unwrap()
            );
            assert_eq!(
                tpe,
                PrimitiveType {
                    name: span2(5, 10, "boolean", files.get(0).unwrap().deref()),
                    tpe: PrimitiveTypeType::Boolean
                }
            );
        }
    }

    #[test]
    fn test_static_class() {
        let (files, root) = apply_semantics!(
            r#"
package dev;

class Test {
  static int a;
  int c;
  void method() {
    int b = Test.a;
    int d = Test.c; // Unable to get the field
  }
}
        "#
        );

        let class = unwrap!(
            CompilationUnitItem::Class,
            &files.first().unwrap().unit.items.get(0).unwrap()
        );
        let method = unwrap!(ClassBodyItem::Method, &class.body.items.get(2).unwrap());
        {
            let var = unwrap!(
                Statement::VariableDeclarators,
                &method.block_opt.as_ref().unwrap().stmts.get(0).unwrap()
            );
            let field_access = unwrap!(
                Expr::FieldAccess,
                var.declarators.first().unwrap().expr_opt.as_ref().unwrap()
            );
            let tpe = unwrap!(
                Type::Primitive,
                field_access.tpe_opt.borrow().clone().unwrap()
            );
            assert_eq!(
                tpe,
                PrimitiveType {
                    name: span2(4, 10, "int", files.get(0).unwrap().deref()),
                    tpe: PrimitiveTypeType::Int
                }
            );
        }
        {
            let var = unwrap!(
                Statement::VariableDeclarators,
                &method.block_opt.as_ref().unwrap().stmts.get(1).unwrap()
            );
            let field_access = unwrap!(
                Expr::FieldAccess,
                var.declarators.first().unwrap().expr_opt.as_ref().unwrap()
            );
            assert_eq!(None, field_access.tpe_opt.borrow().clone());
        }
    }

    #[test]
    fn test_static_parameterized() {
        let (files, root) = apply_semantics!(
            r#"
package dev;

class Test<T extends Test> {
  static int a;
  int c;
  void method() {
    int b = T.a;
    int d = T.c;
  }
}
        "#
        );

        let class = unwrap!(
            CompilationUnitItem::Class,
            &files.first().unwrap().unit.items.get(0).unwrap()
        );
        let method = unwrap!(ClassBodyItem::Method, &class.body.items.get(2).unwrap());
        {
            let var = unwrap!(
                Statement::VariableDeclarators,
                &method.block_opt.as_ref().unwrap().stmts.get(0).unwrap()
            );

            let field_access = unwrap!(
                Expr::FieldAccess,
                var.declarators.first().unwrap().expr_opt.as_ref().unwrap()
            );
            let tpe = unwrap!(
                Type::Primitive,
                field_access.tpe_opt.borrow().clone().unwrap()
            );
            assert_eq!(
                tpe,
                PrimitiveType {
                    name: span2(4, 10, "int", files.get(0).unwrap().deref()),
                    tpe: PrimitiveTypeType::Int
                }
            );
        }
        {
            let var = unwrap!(
                Statement::VariableDeclarators,
                &method.block_opt.as_ref().unwrap().stmts.get(1).unwrap()
            );

            let field_access = unwrap!(
                Expr::FieldAccess,
                var.declarators.first().unwrap().expr_opt.as_ref().unwrap()
            );
            assert_eq!(None, field_access.tpe_opt.borrow().clone());
        }
    }
}
