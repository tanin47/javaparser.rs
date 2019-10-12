use extract::{expr, Definition, Overlay, Usage};
use parse::tree::{FieldAccess, FieldAccessPrefix};

pub fn apply<'def, 'def_ref, 'overlay_ref>(
    field_access: &'def_ref FieldAccess<'def>,
    overlay: &'overlay_ref mut Overlay<'def>,
) {
    apply_prefix(field_access.prefix.borrow().as_ref(), overlay);

    if let Some(field) = field_access.def_opt.borrow().as_ref() {
        overlay.usages.push(Usage {
            span: field_access.name.clone(),
            def: Definition::Field(field.def),
        })
    }
}

pub fn apply_prefix<'def, 'def_ref, 'overlay_ref>(
    prefix: &'def_ref FieldAccessPrefix<'def>,
    overlay: &'overlay_ref mut Overlay<'def>,
) {
    match prefix {
        FieldAccessPrefix::Package(p) => {
            if let Some(span) = p.span_opt {
                overlay.usages.push(Usage {
                    span,
                    def: Definition::Package(p.def),
                })
            }
        }
        FieldAccessPrefix::Expr(e) => expr::apply(e, overlay),
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
    int a;
    void method() {
        Test s;
        int b = s.a;
    }
}
        "#,
            ],
            vec![
                r#"
package dev;

class *0:Test* {
    int *1:a*;
    void *2:method*() {
        [0:Test] *3:s*;
        int *4:b* = [3:s].[1:a];
    }
}
        "#,
            ],
        );
    }
}
