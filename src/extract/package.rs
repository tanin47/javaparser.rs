use extract::Overlay;
use parse::tree::Package;

pub fn apply<'def, 'def_ref, 'overlay_ref>(
    package: &'def_ref Package<'def>,
    overlay: &'overlay_ref mut Overlay<'def>,
) {
    // TODO: decide what to do with package declaration.
}
