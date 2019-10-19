use analyze;
use analyze::definition::{Param, TypeParam};
use analyze::resolve::scope::Scope;
use parse::tree::{ClassType, Lambda, Name, ParameterizedType, Type};
use semantics::expr::method_call::coerce_from_primitive_to_class;
use semantics::{block, expr, Context};
use std::cell::RefCell;
use std::ops::Deref;

pub fn apply<'def>(
    lambda: &mut Lambda<'def>,
    target_type: &Type<'def>,
    context: &mut Context<'def, '_, '_>,
) {
    lambda.inferred_class_type_opt = if let Type::Class(c) = target_type {
        Some(c.clone())
    } else {
        None
    };
    lambda.inferred_method_opt = lambda
        .inferred_class_type_opt
        .as_ref()
        .and_then(|c| c.lambda_method());

    context.scope.enter();

    if let Some(method) = &mut lambda.inferred_method_opt {
        for (param, target_param) in lambda.params.iter().zip(method.params.iter_mut()) {
            target_param.tpe.replace(infer(
                &param.tpe,
                target_param.tpe.borrow().deref(),
                context,
            ));
            context.scope.add_param(target_param);
        }
    }

    for param in &lambda.inferred_params {
        context.scope.add_param(param);
    }

    if let Some(e) = &mut lambda.expr_opt {
        expr::apply(e, &Type::UnknownType, context);

        if let Some(method) = &mut lambda.inferred_method_opt {
            method.return_type = e.tpe_opt().unwrap_or(Type::UnknownType);
        }
    } else if let Some(b) = &mut lambda.block_opt {
        block::apply(b, context);

        lambda.inferred_return_type = b.return_type.clone();
    }

    if let Some(method) = &lambda.inferred_method_opt {
        lambda.inferred_return_type =
            infer(&lambda.inferred_return_type, &method.return_type, context);
    }

    // TODO: we need to set the ClassType correctly here.

    context.scope.leave();
}

fn infer<'def>(
    declared: &Type<'def>,
    target: &Type<'def>,
    context: &Context<'def, '_, '_>,
) -> Type<'def> {
    match declared {
        Type::Class(declared) => return Type::Class(infer_class(declared, target)),
        Type::Primitive(p) => return Type::Primitive(p.clone()),
        Type::Parameterized(declared) => return infer_parameterized(declared, target, context),
        Type::Array(_) => (),
        Type::Wildcard(_) => (),
        Type::UnknownType => return target.clone(),
        Type::Void(_) => panic!(),
        Type::Lambda(_) => panic!(),
    }

    declared.clone()
}

fn infer_parameterized<'def>(
    declared: &ParameterizedType<'def>,
    target: &Type<'def>,
    context: &Context<'def, '_, '_>,
) -> Type<'def> {
    match target {
        Type::Primitive(p) => return Type::Class(coerce_from_primitive_to_class(p, context)),
        Type::Class(c) => return Type::Class(c.clone()),
        _ => (),
    };

    Type::Parameterized(declared.clone())
}

fn infer_class<'def>(declared: &ClassType<'def>, target: &Type<'def>) -> ClassType<'def> {
    match target {
        Type::Class(target) => {
            let equal_with_def = declared.def_opt.is_some() && declared.def_opt == target.def_opt;
            let equal_without_def = declared.def_opt.is_none()
                && target.def_opt.is_none()
                && declared.name == target.name;
            if equal_with_def || equal_without_def {
                // TODO: we need to recursively handle prefix_opt and type_args_opt
                return ClassType {
                    prefix_opt: None,
                    name: declared.name.clone(),
                    span_opt: declared.span_opt,
                    // TODO: what do we do if `target` has `type_args` but `declared` doesn't? And vice versa
                    // TODO: what do we do if they have different numbers of `type_args`?
                    type_args_opt: None,
                    def_opt: declared.def_opt,
                };
            }
        }
        _ => (),
    }

    declared.clone()
}

#[cfg(test)]
mod tests {
    use analyze::definition::Param;
    use parse::tree::{
        ClassBodyItem, ClassType, CompilationUnitItem, Expr, PrimitiveType, PrimitiveTypeType,
        ResolvedName, Statement, Type, TypeParam,
    };
    use std::cell::RefCell;
    use std::ops::Deref;
    use test_common::span2;

    #[test]
    fn test_simple() {
        let (files, root) = apply_semantics!(
            r#"
package dev;

class Test {
  void method() {
    method((a) -> { return a; });
  }
  
  void method(Function fn) {}
}
        "#,
            r#"
package dev;

abstract class Function {
  int main(int a);
}
        "#
        );

        let class = unwrap!(
            CompilationUnitItem::Class,
            &files.first().unwrap().unit.items.get(0).unwrap()
        );
        let method = unwrap!(ClassBodyItem::Method, &class.body.items.get(0).unwrap());
        let expr_stmt = unwrap!(
            Statement::Expr,
            &method.block_opt.as_ref().unwrap().stmts.get(0).unwrap()
        );
        let method_call = unwrap!(Expr::MethodCall, &expr_stmt);

        let lambda = unwrap!(Expr::Lambda, method_call.args.get(0).unwrap());

        let return_stmt = unwrap!(
            Statement::Return,
            lambda.block_opt.as_ref().unwrap().stmts.get(0).unwrap()
        );
        let return_expr = unwrap!(Expr::Name, return_stmt.expr_opt.as_ref().unwrap());

        let param =
            unsafe { &*unwrap!(ResolvedName::Param, return_expr.resolved_opt.get().unwrap()) };

        assert_eq!(
            param.tpe.borrow().deref(),
            &Type::Primitive(PrimitiveType {
                span_opt: Some(span2(4, 12, "int", files.get(1).unwrap().deref())),
                tpe: PrimitiveTypeType::Int
            })
        )
    }

    #[test]
    fn test_infer_from_return_statement() {
        let (files, root) = apply_semantics!(
            r#"
package dev;

class Test {
  void method() {
    method(() -> { return 3; });
  }
  
  <T> T method(Function<T> fn) {}
}
        "#,
            r#"
package dev;

abstract class Function<T> {
  T main();
}
        "#
        );

        let class = unwrap!(
            CompilationUnitItem::Class,
            &files.first().unwrap().unit.items.get(0).unwrap()
        );
        let method = unwrap!(ClassBodyItem::Method, &class.body.items.get(0).unwrap());
        let expr_stmt = unwrap!(
            Statement::Expr,
            &method.block_opt.as_ref().unwrap().stmts.get(0).unwrap()
        );
        let method_call = unwrap!(Expr::MethodCall, &expr_stmt);

        assert_eq!(
            &method_call
                .def_opt
                .borrow()
                .deref()
                .as_ref()
                .unwrap()
                .return_type,
            &Type::Class(ClassType {
                prefix_opt: None,
                name: "Integer".to_string(),
                span_opt: None,
                type_args_opt: None,
                def_opt: None
            })
        );
    }
}
