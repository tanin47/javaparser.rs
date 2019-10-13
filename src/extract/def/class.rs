use extract::def::{field, method, type_param};
use extract::{Definition, Overlay};
use parse::tree::{Class, ClassBodyItem};

pub fn apply<'def, 'def_ref, 'overlay_ref>(
    class: &'def_ref Class<'def>,
    overlay: &'overlay_ref mut Overlay<'def>,
) {
    if let Some(def) = class.def_opt.borrow().as_ref() {
        overlay.defs.push(Definition::Class(*def));
    }

    for t in &class.type_params {
        type_param::apply(t, overlay);
    }

    for item in &class.body.items {
        apply_item(item, overlay);
    }
}

pub fn apply_item<'def, 'def_ref, 'overlay_ref>(
    item: &'def_ref ClassBodyItem<'def>,
    overlay: &'overlay_ref mut Overlay<'def>,
) {
    match item {
        ClassBodyItem::Method(m) => method::apply(m, overlay),
        ClassBodyItem::FieldDeclarators(f) => field::apply(f, overlay),
        ClassBodyItem::Class(c) => apply(c, overlay),
        ClassBodyItem::Interface(_) => {}
        ClassBodyItem::Enum(_) => {}
        ClassBodyItem::Annotation(_) => {}
        ClassBodyItem::StaticInitializer(_) => {}
        ClassBodyItem::Constructor(_) => {}
    };
}
