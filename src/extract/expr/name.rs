use extract::{expr, Definition, Overlay, Usage};
use parse::tree::{FieldAccess, FieldAccessPrefix, Name, ResolvedName};

pub fn apply<'def, 'def_ref, 'overlay_ref>(
    name: &'def_ref Name<'def>,
    overlay: &'overlay_ref mut Overlay<'def>,
) {
    let resolved = if let Some(resolved) = name.resolved_opt.get() {
        resolved
    } else {
        return;
    };

    let def = match resolved {
        ResolvedName::Package(p) => Definition::Package(p),
        ResolvedName::Class(c) => Definition::Class(c),
        ResolvedName::Variable(v) => Definition::VariableDeclarator(v),
        ResolvedName::TypeParam(t) => Definition::TypeParam(t),
    };

    overlay.usages.push(Usage {
        span: name.name,
        def,
    })
}
