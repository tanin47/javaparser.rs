use extract::{Definition, Overlay, Usage};
use parse::tree::{ClassType, ParameterizedType};

pub fn apply<'def, 'def_ref, 'overlay_ref>(
    parameterized: &'def_ref ParameterizedType<'def>,
    overlay: &'overlay_ref mut Overlay<'def>,
) {
    let def = unsafe { &*parameterized.def };

    if let Some(span) = &parameterized.span_opt {
        overlay.usages.push(Usage {
            span: span.clone(),
            def: Definition::TypeParam(def),
        })
    }
}
