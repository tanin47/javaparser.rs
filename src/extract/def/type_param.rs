use extract::{block, tpe, Definition, Overlay};
use parse::tree::{
    FieldDeclarator, FieldDeclarators, TypeParam, TypeParamExtend, VariableDeclarator,
};

pub fn apply<'def, 'def_ref, 'overlay_ref>(
    type_param: &'def_ref TypeParam<'def>,
    overlay: &'overlay_ref mut Overlay<'def>,
) {
    if let Some(def) = type_param.def_opt.borrow().as_ref() {
        overlay.defs.push(Definition::TypeParam(*def));
    }

    for extend in &type_param.extends {
        match extend {
            TypeParamExtend::Class(c) => tpe::class::apply(c, overlay),
            TypeParamExtend::Parameterized(p) => tpe::parameterized::apply(p, overlay),
        };
    }
}
