use extract::Overlay;
use parse::tree::Type;

pub mod class;

pub fn apply<'def, 'def_ref, 'overlay_ref>(
    tpe: &'def_ref Type<'def>,
    overlay: &'overlay_ref mut Overlay<'def>,
) {
    match tpe {
        Type::Class(c) => class::apply(c, overlay),
        Type::Primitive(_) => {}
        Type::Array(_) => {}
        Type::Wildcard(_) => {}
        Type::Parameterized(_) => {}
        Type::Void(_) => {}
        Type::UnknownType => {}
    }
}
