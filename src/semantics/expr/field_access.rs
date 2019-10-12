use analyze::resolve::scope::Scope;
use parse::tree::{
    EnclosingType, Expr, FieldAccess, FieldAccessPrefix, InvocationContext, PackagePrefix,
    ParameterizedType, ResolvedName, StaticClass, StaticType,
};
use semantics::{expr, Context};
use std::ops::Deref;

pub fn apply<'def, 'def_ref>(
    field_access: &'def_ref FieldAccess<'def>,
    context: &mut Context<'def, 'def_ref, '_>,
) {
    apply_field_access(field_access, context);
}

pub fn apply_field_access<'def, 'def_ref>(
    field_access: &'def_ref FieldAccess<'def>,
    context: &mut Context<'def, 'def_ref, '_>,
) -> Option<FieldAccessPrefix<'def>> {
    let new_prefix_opt = apply_prefix(
        unsafe { field_access.prefix.try_borrow_unguarded() }.unwrap(),
        context,
    );

    if let Some(new_prefix) = new_prefix_opt {
        field_access.prefix.replace(Box::new(new_prefix));
    }

    match field_access.prefix.borrow().as_ref() {
        FieldAccessPrefix::Package(p) => {
            if let Some(mut tpe) = p.find(field_access.name.fragment) {
                tpe.set_span_opt(Some(&field_access.name));
                match tpe {
                    EnclosingType::Package(p) => return Some(FieldAccessPrefix::Package(p)),
                    EnclosingType::Class(c) => {
                        return Some(FieldAccessPrefix::Expr(Expr::StaticClass(StaticClass {
                            tpe: StaticType::Class(c),
                        })));
                    }
                    EnclosingType::Parameterized(p) => {
                        return Some(FieldAccessPrefix::Expr(Expr::StaticClass(StaticClass {
                            tpe: StaticType::Parameterized(p),
                        })));
                    }
                }
            }
        }
        FieldAccessPrefix::Expr(Expr::StaticClass(static_class)) => {
            if let Some(mut class) = static_class
                .tpe
                .find_inner_class(field_access.name.fragment)
            {
                class.set_span_opt(Some(&field_access.name));
                return Some(FieldAccessPrefix::Expr(Expr::StaticClass(StaticClass {
                    tpe: StaticType::Class(class),
                })));
            }
            if let Some(field) = static_class.tpe.find_field(
                field_access.name.fragment,
                &InvocationContext { only_static: true },
            ) {
                field_access.def_opt.replace(Some(field));
            }
        }
        FieldAccessPrefix::Expr(e) => {
            if let Some(tpe) = e.tpe_opt() {
                if let Some(field) = tpe.find_field(
                    field_access.name.fragment,
                    &InvocationContext { only_static: false },
                ) {
                    field_access.def_opt.replace(Some(field));
                }
            }
        }
    }

    None
}

fn apply_prefix<'def, 'def_ref>(
    prefix: &'def_ref FieldAccessPrefix<'def>,
    context: &mut Context<'def, 'def_ref, '_>,
) -> Option<FieldAccessPrefix<'def>> {
    let ex = match prefix {
        FieldAccessPrefix::Expr(e) => e,
        // The below only happens when we process the node more than once.
        FieldAccessPrefix::Package(p) => return None,
    };

    match ex {
        Expr::FieldAccess(f) => {
            if let Some(new_prefix) = apply_field_access(f, context) {
                return Some(new_prefix);
            }
        }
        Expr::Name(n) => {
            if let Some(resolved) = context.scope.resolve_name(n.name.fragment) {
                match resolved {
                    ResolvedName::Package(p) => {
                        return Some(FieldAccessPrefix::Package(PackagePrefix {
                            prefix_opt: None,
                            name: n.name.fragment.to_owned(),
                            span_opt: Some(n.name.clone()),
                            def: p,
                        }))
                    }
                    ResolvedName::Class(c) => {
                        return Some(FieldAccessPrefix::Expr(Expr::StaticClass(StaticClass {
                            tpe: StaticType::Class({
                                let mut t = unsafe { &*c }.to_type();
                                t.set_span_opt(Some(&n.name));
                                t
                            }),
                        })))
                    }
                    ResolvedName::TypeParam(p) => {
                        return Some(FieldAccessPrefix::Expr(Expr::StaticClass(StaticClass {
                            tpe: StaticType::Parameterized(ParameterizedType {
                                name: n.name.fragment.to_owned(),
                                span_opt: Some(n.name.clone()),
                                def: p,
                            }),
                        })))
                    }
                    other => n.resolved_opt.set(Some(other)),
                };
            }
        }
        other => expr::apply(other, context),
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
    fn test_array_field() {
        let (files, root) = apply_semantics!(
            r#"
package dev;

class Test {
  void method() {
    int[] s;
    int a = s.length;
  }
}
        "#
        );

        let class = unwrap!(
            CompilationUnitItem::Class,
            &files.first().unwrap().unit.items.get(0).unwrap()
        );
        let method = unwrap!(ClassBodyItem::Method, &class.body.items.get(0).unwrap());
        let var = unwrap!(
            Statement::VariableDeclarators,
            &method.block_opt.as_ref().unwrap().stmts.get(1).unwrap()
        );

        let field_access = unwrap!(
            Expr::FieldAccess,
            var.declarators.first().unwrap().expr_opt.as_ref().unwrap()
        );
        println!("{:#?}", field_access);
        let tpe = unwrap!(
            Type::Primitive,
            field_access.def_opt.borrow().as_ref().unwrap().tpe.clone()
        );
        assert_eq!(
            tpe,
            PrimitiveType {
                span_opt: None,
                tpe: PrimitiveTypeType::Int
            }
        );
    }

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
                field_access.def_opt.borrow().as_ref().unwrap().tpe.clone()
            );
            assert_eq!(
                tpe,
                PrimitiveType {
                    span_opt: Some(span2(4, 3, "int", files.get(1).unwrap().deref())),
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
                field_access.def_opt.borrow().as_ref().unwrap().tpe.clone()
            );
            assert_eq!(
                tpe,
                PrimitiveType {
                    span_opt: Some(span2(5, 10, "boolean", files.get(0).unwrap().deref())),
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
    int e = dev.Test.a;
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
                field_access.def_opt.borrow().as_ref().unwrap().tpe.clone()
            );
            assert_eq!(
                tpe,
                PrimitiveType {
                    span_opt: Some(span2(4, 10, "int", files.get(0).unwrap().deref())),
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
            assert_eq!(None, field_access.def_opt.borrow().as_ref());
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
                field_access.def_opt.borrow().as_ref().unwrap().tpe.clone()
            );
            assert_eq!(
                tpe,
                PrimitiveType {
                    span_opt: Some(span2(4, 10, "int", files.get(0).unwrap().deref())),
                    tpe: PrimitiveTypeType::Int
                }
            );
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
                field_access.def_opt.borrow().as_ref().unwrap().tpe.clone()
            );
            assert_eq!(
                tpe,
                PrimitiveType {
                    span_opt: Some(span2(4, 10, "int", files.get(0).unwrap().deref())),
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
            assert_eq!(None, field_access.def_opt.borrow().as_ref());
        }
    }

    #[test]
    fn test_super() {
        let (files, root) = apply_semantics!(
            r#"
package dev;

class Test extends Super {
  void method() {
    int b = Test.a;
    int d = Test.c;
  }
}
        "#,
            r#"
package dev;

class Super {
  static int a;
  int c;
}
        "#
        );

        let class = unwrap!(
            CompilationUnitItem::Class,
            &files.first().unwrap().unit.items.get(0).unwrap()
        );
        let method = unwrap!(ClassBodyItem::Method, &class.body.items.get(0).unwrap());
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
                field_access.def_opt.borrow().as_ref().unwrap().tpe.clone()
            );
            assert_eq!(
                tpe,
                PrimitiveType {
                    span_opt: Some(span2(4, 10, "int", files.get(1).unwrap().deref())),
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
            assert_eq!(None, field_access.def_opt.borrow().as_ref());
        }
    }
}
