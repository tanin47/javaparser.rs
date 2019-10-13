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
    fn test_var() {
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

    #[test]
    fn test_static_class() {
        assert_extract(
            vec![
                r#"
package dev.other;

class Test {
    static class Inner {
      static int inner;
    }
    static int a;
    void method() {
        int b = Test.a;
        int c = dev.other.Test.a;
        int d = dev.other.Test.Inner.inner;
    }
}
        "#,
            ],
            vec![
                r#"
package dev.other;

class *0:Test* {
    static class *1:Inner* {
      static int *2:inner*;
    }
    static int *3:a*;
    void *4:method*() {
        int *5:b* = [0:Test].[3:a];
        int *6:c* = [dev].[other].[0:Test].[3:a];
        int *7:d* = [dev].[other].[0:Test].[1:Inner].[2:inner];
    }
}
        "#,
            ],
        );
    }

    #[test]
    fn test_static_parameterized() {
        assert_extract(
            vec![
                r#"
package dev;

class Test<T extends Test> {
    static class Inner {
      static int inner;
    }
    static int a;
    void method() {
        int b = T.a;
        int c = T.Inner.inner;
    }
}
        "#,
            ],
            vec![
                r#"
package dev;

class *0:Test*<*1:T* extends [0:Test]> {
    static class *2:Inner* {
      static int *3:inner*;
    }
    static int *4:a*;
    void *5:method*() {
        int *6:b* = [1:T].[4:a];
        int *7:c* = [1:T].[2:Inner].[3:inner];
    }
}
        "#,
            ],
        );
    }

    #[test]
    fn test_array() {
        assert_extract(
            vec![
                r#"
package dev;

class Test {
    void method() {
        int[] a;
        int b = a.length;
    }
}
        "#,
            ],
            vec![
                r#"
package dev;

class *0:Test* {
    void *1:method*() {
        int[] *2:a*;
        int *3:b* = [2:a].[length];
    }
}
        "#,
            ],
        );
    }
}
