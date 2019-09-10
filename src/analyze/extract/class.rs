use analyze::definition::Class;
use analyze::extract::{Extraction, Usage};
use analyze::resolve::scope::Scope;
use std::intrinsics::offset;

pub fn apply<'def, 'def_ref>(
    class: &Class<'def>,
    extraction: &mut Extraction,
    scope: &mut Scope<'def, 'def_ref>,
) {
    scope.enter();
    for type_param in &class.type_params {
        scope.add_type_param(type_param);
    }

    if let Some(extend) = class.extend_opt.borrow().as_ref() {
        if let Some(def) = extend.def_opt.get() {
            let def = unsafe { &(*def) };
            // TODO: we need the extend's and definition's location.
            extraction.usages.push(Usage {})
        }
    }

    scope.enter_class(class);
    for type_param in &class.type_params {
        scope.add_type_param(type_param);
    }

    scope.leave();
    scope.leave();
}
