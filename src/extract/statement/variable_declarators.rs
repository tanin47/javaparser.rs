use extract::{expr, tpe, Definition, Overlay, Usage};
use parse::tree::{VariableDeclarator, VariableDeclarators};
use std::ops::Deref;

pub fn apply<'def, 'def_ref, 'overlay_ref>(
    variable_declarators: &'def_ref VariableDeclarators<'def>,
    overlay: &'overlay_ref mut Overlay<'def>,
) {
    for decl in &variable_declarators.declarators {
        apply_decl(decl, overlay);
    }
}

fn apply_decl<'def, 'def_ref, 'overlay_ref>(
    decl: &'def_ref VariableDeclarator<'def>,
    overlay: &'overlay_ref mut Overlay<'def>,
) {
    overlay.defs.push(Definition::VariableDeclarator(decl));

    tpe::apply(decl.tpe.borrow().deref(), overlay);

    if let Some(e) = &decl.expr_opt {
        expr::apply(e, overlay);
    }
}

#[cfg(test)]
mod tests {
    use extract::test_common::assert_extract;

    #[test]
    fn test() {
        assert_extract(
            vec![
                r#"
package dev;

class Test {
    void method() {
        Test s; 
    }
}
        "#,
            ],
            vec![
                r#"
package dev;

class *0:Test* {
    void *1:method*() {
        [0:Test] *2:s*; 
    }
}
        "#,
            ],
        );
    }
}
