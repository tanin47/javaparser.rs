use extract::{import, package, Overlay};
use parse::tree::CompilationUnit;

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
}
