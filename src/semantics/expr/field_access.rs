use analyze::resolve::scope::Scope;
use parse::tree::FieldAccess;
use semantics::expr;

pub fn apply<'def, 'def_ref, 'scope_ref>(
    field_access: &'def_ref FieldAccess<'def>,
    scope: &'scope_ref mut Scope<'def, 'def_ref>,
) {
    expr::apply(&field_access.expr, scope);

    if let Some(tpe) = field_access.expr.tpe_opt() {
        field_access
            .tpe_opt
            .replace(tpe.find(field_access.field.name.fragment));
    }
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
    fn test_concrete() {
        let (files, root) = apply_semantics!(
            r#"
package dev;

class Test<T> {
  T a;
  void method() {
    Test<Another> s; 
    int b = s.a.num;
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
        let method = unwrap!(ClassBodyItem::Method, &class.body.items.get(1).unwrap());
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
}
