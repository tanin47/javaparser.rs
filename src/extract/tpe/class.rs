use extract::tpe::{package, parameterized};
use extract::{Definition, Overlay, Usage};
use parse::tree::{ClassType, EnclosingType};

pub fn apply<'def, 'def_ref, 'overlay_ref>(
    class: &'def_ref ClassType<'def>,
    overlay: &'overlay_ref mut Overlay<'def>,
) {
    if let Some(prefix) = &class.prefix_opt {
        match prefix.as_ref() {
            EnclosingType::Package(p) => package::apply(p, overlay),
            EnclosingType::Class(c) => apply(c, overlay),
            EnclosingType::Parameterized(p) => parameterized::apply(p, overlay),
        }
    }

    if let Some(def) = class.def_opt {
        if let Some(span) = &class.span_opt {
            overlay.usages.push(Usage {
                span: span.clone(),
                def: Definition::Class(def),
            })
        }
    }
}
