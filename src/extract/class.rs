use extract::{method, Definition, Overlay};
use parse::tree::{Class, ClassBodyItem};

pub fn apply<'def, 'def_ref, 'overlay_ref>(
    class: &'def_ref Class<'def>,
    overlay: &'overlay_ref mut Overlay<'def>,
) {
    overlay.defs.push(Definition::Class(class));

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
        ClassBodyItem::FieldDeclarators(_) => {}
        ClassBodyItem::Class(_) => {}
        ClassBodyItem::Interface(_) => {}
        ClassBodyItem::Enum(_) => {}
        ClassBodyItem::Annotation(_) => {}
        ClassBodyItem::StaticInitializer(_) => {}
        ClassBodyItem::Constructor(_) => {}
    };
}
