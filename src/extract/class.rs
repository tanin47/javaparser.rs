use extract::{Definition, Overlay};
use parse::tree::Class;

pub fn apply<'def, 'def_ref, 'overlay_ref>(
    class: &'def_ref Class<'def>,
    overlay: &'overlay_ref mut Overlay<'def>,
) {
    if let Some(def) = class.def_opt.borrow().as_ref() {
        overlay.defs.push(Definition::Class(*def));
    }
}
