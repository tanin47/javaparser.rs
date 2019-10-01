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
                def: Definition::Class(*class),
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
                def: Definition::Class(*class),
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
    use analyze::test_common::{find_class, find_package, make_root, make_tokenss, make_units};
    use extract;
    use parse::tree::{Import, ImportDef, ImportPrefix, ImportPrefixDef};
    use std::cell::RefCell;
    use test_common::span;
    use {analyze, semantics};

    #[test]
    fn test() {
        let raws = vec![
            r#"
package dev;

import dev2.Super;
import static dev2.*;

class Test {}
        "#
            .to_owned(),
            r#"
package dev2;

class Super {}
        "#
            .to_owned(),
        ];
        let tokenss = make_tokenss(&raws);
        let units = make_units(&tokenss);
        let root = analyze::resolve::apply(&units);

        semantics::apply(units.first().unwrap(), &root);
        let overlay = extract::apply(units.first().unwrap());

        println!("{:#?}", overlay);
    }
}
