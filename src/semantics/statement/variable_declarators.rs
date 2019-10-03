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
    use {analyze, semantics};

    #[test]
    fn test_concrete() {
        let (files, _) = semantics_files![
            r#"
package dev;

class Test<T> {
  void method() {
    T s; 
    s = null;
  }
}
        "#
        ];

        println!("{:#?}", files.first().unwrap().unit);
    }
}
