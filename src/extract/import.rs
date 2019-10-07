use extract::{Definition, Overlay, Usage};
use parse::tree::{Import, ImportDef, ImportPrefix, ImportPrefixDef};
use std::ops::Deref;

pub fn apply<'def, 'def_ref, 'overlay_ref>(
    import: &'def_ref Import<'def>,
    overlay: &'overlay_ref mut Overlay<'def>,
) {
    if let Some(prefix) = &import.prefix_opt {
        apply_prefix(prefix, overlay);
    }

    if let Some(def) = import.def_opt.borrow().as_ref() {
        match def {
            ImportDef::Class(class) => overlay.usages.push(Usage {
                span: import.name.clone(),
                def: Definition::Class(unsafe { &**class }.parse),
            }),
            ImportDef::Package(p) => overlay.usages.push(Usage {
                span: import.name.clone(),
                def: Definition::Package(*p),
            }),
        }
    }
}
fn apply_prefix<'def, 'def_ref, 'overlay_ref>(
    import: &'def_ref ImportPrefix<'def>,
    overlay: &'overlay_ref mut Overlay<'def>,
) {
    if let Some(def) = import.def_opt.borrow().as_ref() {
        match def {
            ImportPrefixDef::Class(class) => overlay.usages.push(Usage {
                span: import.name.clone(),
                def: Definition::Class(unsafe { &**class }.parse),
            }),
            ImportPrefixDef::Package(p) => overlay.usages.push(Usage {
                span: import.name.clone(),
                def: Definition::Package(*p),
            }),
        }
    }

    if let Some(prefix) = &import.prefix_opt {
        apply_prefix(prefix, overlay);
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

import dev2.Super;
import static dev2.*;

class Test {}
        "#,
                r#"
package dev2;

class Super {}
        "#,
            ],
            vec![
                r#"
package dev;

import [dev2].[1:Super];
import static [dev2].*;

class *0:Test* {}
        "#,
                r#"
package dev2;

class *1:Super* {}
        "#,
            ],
        );
    }
}
