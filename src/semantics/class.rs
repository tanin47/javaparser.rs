use analyze::resolve::scope::Scope;
use parse;
use parse::tree::{Class, ClassBodyItem, EnclosingType};
use semantics::method;
use std::borrow::Borrow;

pub fn apply<'def, 'def_ref, 'scope_ref>(
    class: &'def_ref parse::tree::Class<'def>,
    scope: &'scope_ref mut Scope<'def, 'def_ref>,
) {
    if let Some(EnclosingType::Class(tpe)) = scope.resolve_type(&class.name) {
        if let Some(def) = tpe.def_opt {
            class.def_opt.replace(Some(def));
        }
    }

    if let Some(def) = class.def_opt.borrow().as_ref() {
        scope.enter_class(unsafe { &**def });
    } else {
        scope.enter();
    }

    scope.leave();
}

fn apply_class_body<'def, 'def_ref, 'scope_ref>(
    body: &'def_ref parse::tree::ClassBody<'def>,
    scope: &'scope_ref mut Scope<'def, 'def_ref>,
) {
    for item in &body.items {
        match item {
            ClassBodyItem::Method(m) => method::apply(m, scope),
            _ => (),
        };
    }
}
