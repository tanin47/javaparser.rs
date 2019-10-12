use analyze::resolve;
use analyze::resolve::scope::Scope;
use parse::tree::{VariableDeclarator, VariableDeclarators};
use semantics::{expr, Context};

pub fn apply<'def, 'def_ref, 'scope_ref>(
    declarator: &'def_ref VariableDeclarators<'def>,
    context: &mut Context<'def, 'def_ref, '_>,
) {
    for decl in &declarator.declarators {
        let resolved = resolve::apply_type(&decl.tpe.borrow(), &mut context.scope);
        decl.tpe.replace(resolved);

        context.scope.add_variable(decl);

        if let Some(ex) = &decl.expr_opt {
            expr::apply(ex, context);
        }
    }
}

#[cfg(test)]
mod tests {
    use analyze::test_common::find_class;
    use parse::tree::{
        ArrayType, ClassBodyItem, ClassType, CompilationUnitItem, ParameterizedType, Statement,
        Type, TypeArg, TypeParam, NATIVE_ARRAY_CLASS_NAME,
    };
    use std::ops::Deref;
    use test_common::span2;
    use {analyze, semantics};

    #[test]
    fn test_concrete() {
        let (files, root) = apply_semantics!(
            r#"
package dev;

class Test<T> {
  void method() {
    T s; 
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
            &method.block_opt.as_ref().unwrap().stmts.get(0).unwrap()
        );
        let tpe = unwrap!(
            Type::Parameterized,
            var.declarators
                .first()
                .unwrap()
                .tpe
                .borrow()
                .deref()
                .clone()
        );
        assert_eq!(
            tpe.def,
            find_class(&root, "dev.Test").type_params.first().unwrap()
        );
    }

    #[test]
    fn test_array() {
        let (files, root) = apply_semantics!(
            r#"
package dev;

class Test<T> {
  void method() {
    T[] s;
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
            &method.block_opt.as_ref().unwrap().stmts.get(0).unwrap()
        );
        let tpe = unwrap!(
            Type::Array,
            var.declarators
                .first()
                .unwrap()
                .tpe
                .borrow()
                .deref()
                .clone()
        );
        assert_eq!(
            tpe,
            ArrayType {
                tpe: Box::new(Type::Parameterized(ParameterizedType {
                    name: "T".to_owned(),
                    span_opt: Some(span2(5, 5, "T", files.get(0).unwrap().deref())),
                    def: find_class(&root, "dev.Test").type_params.first().unwrap(),
                })),
                size_opt: None,
                underlying: ClassType {
                    prefix_opt: None,
                    name: NATIVE_ARRAY_CLASS_NAME.to_owned(),
                    span_opt: None,
                    type_args_opt: Some(vec![TypeArg::Parameterized(ParameterizedType {
                        name: "T".to_owned(),
                        span_opt: Some(span2(5, 5, "T", files.get(0).unwrap().deref())),
                        def: find_class(&root, "dev.Test").type_params.first().unwrap(),
                    })]),
                    def_opt: Some(find_class(&root, NATIVE_ARRAY_CLASS_NAME))
                }
            }
        );
    }
}
