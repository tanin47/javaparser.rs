use analyze::resolve;
use analyze::resolve::scope::Scope;
use parse::tree::{VariableDeclarator, VariableDeclarators};

pub fn apply<'def, 'def_ref, 'scope_ref>(
    declarator: &'def_ref VariableDeclarators<'def>,
    scope: &'scope_ref mut Scope<'def, 'def_ref>,
) {
    for decl in &declarator.declarators {
        let resolved = resolve::apply_type(&decl.tpe.borrow(), scope);
        decl.tpe.replace(resolved);
    }
}

#[cfg(test)]
mod tests {
    use analyze::test_common::{make_tokenss, make_units};
    use {analyze, semantics};

    #[test]
    fn test_concrete() {
        let raws = vec![r#"
package dev;

class Test<T> {
  void method() {
    T s; 
    s = null;
  }
}
        "#
        .to_owned()];
        let tokenss = make_tokenss(&raws);
        let units = make_units(&tokenss);
        let root = analyze::resolve::apply(&units);

        semantics::apply(units.first().unwrap(), &root);

        println!("{:#?}", units.first().unwrap());
    }
}
