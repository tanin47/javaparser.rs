use extract::{block, Definition, Overlay};
use parse::tree::{FieldDeclarator, FieldDeclarators, VariableDeclarator};

pub fn apply<'def, 'def_ref, 'overlay_ref>(
    field_declarators: &'def_ref FieldDeclarators<'def>,
    overlay: &'overlay_ref mut Overlay<'def>,
) {
    for decl in &field_declarators.declarators {
        apply_decl(decl, overlay)
    }
}

pub fn apply_decl<'def, 'def_ref, 'overlay_ref>(
    decl: &'def_ref FieldDeclarator<'def>,
    overlay: &'overlay_ref mut Overlay<'def>,
) {
    if let Some(def) = decl.def_opt.borrow().as_ref() {
        overlay.defs.push(Definition::Field(*def));
    }
}
