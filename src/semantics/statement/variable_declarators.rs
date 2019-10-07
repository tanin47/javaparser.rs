use analyze::resolve;
use analyze::resolve::scope::Scope;
use parse::tree::{VariableDeclarator, VariableDeclarators};
use semantics::expr;

pub fn apply<'def, 'def_ref, 'scope_ref>(
    declarator: &'def_ref VariableDeclarators<'def>,
    scope: &'scope_ref mut Scope<'def, 'def_ref>,
) {
    for decl in &declarator.declarators {
        let resolved = resolve::apply_type(&decl.tpe.borrow(), scope);
        decl.tpe.replace(resolved);

        scope.add_variable(decl);

        if let Some(ex) = &decl.expr_opt {
            expr::apply(ex, scope);
        }
    }
}

#[cfg(test)]
mod tests {
    use analyze::test_common::find_class;
    use parse::tree::{ClassBodyItem, CompilationUnitItem, Statement, Type, TypeParam};
    use std::ops::Deref;
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
}
