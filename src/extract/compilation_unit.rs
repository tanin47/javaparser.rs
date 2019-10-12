use extract::def::{class, package};
use extract::{import, Overlay};
use parse::tree::{CompilationUnit, CompilationUnitItem};

pub fn apply<'def, 'def_ref, 'overlay_ref>(
    unit: &'def_ref CompilationUnit<'def>,
    overlay: &'overlay_ref mut Overlay<'def>,
) {
    if let Some(pck) = &unit.package_opt {
        package::apply(pck, overlay);
    }

    for im in &unit.imports {
        import::apply(im, overlay);
    }

    for item in &unit.items {
        apply_item(item, overlay);
    }
}

pub fn apply_item<'def, 'def_ref, 'overlay_ref>(
    item: &'def_ref CompilationUnitItem<'def>,
    overlay: &'overlay_ref mut Overlay<'def>,
) {
    match item {
        CompilationUnitItem::Class(c) => class::apply(c, overlay),
        CompilationUnitItem::Interface(_) => (),
        CompilationUnitItem::Annotation(_) => (),
        CompilationUnitItem::Enum(_) => (),
    };
}
