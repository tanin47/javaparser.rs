use extract::{Definition, Overlay, Usage};
use parse::tree::ClassType;

pub fn apply<'def, 'def_ref, 'overlay_ref>(
    class: &'def_ref ClassType<'def>,
    overlay: &'overlay_ref mut Overlay<'def>,
) {
    if let Some(def) = class.def_opt {
        overlay.usages.push(Usage {
            span: class.name,
            def: Definition::Class(unsafe { &*def }.parse),
        })
    }
}
