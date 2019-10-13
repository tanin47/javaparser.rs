use extract::{expr, tpe, Definition, Overlay, Usage};
use parse::tree::{FieldAccess, FieldAccessPrefix, Name, ResolvedName, StaticClass, StaticType};

pub fn apply<'def, 'def_ref, 'overlay_ref>(
    static_class: &'def_ref StaticClass<'def>,
    overlay: &'overlay_ref mut Overlay<'def>,
) {
    match &static_class.tpe {
        StaticType::Class(c) => tpe::class::apply(c, overlay),
        StaticType::Parameterized(p) => tpe::parameterized::apply(p, overlay),
    }
}
