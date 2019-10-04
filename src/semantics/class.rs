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

    scope.enter();
    // TypeParam can be referred to in the 'extend' section. But the class itself can't.
    // So, we do double-scope here.
    if let Some(def) = class.def_opt.borrow().as_ref() {
        let def = unsafe { &**def };
        for type_param in &def.type_params {
            scope.add_type_param(type_param);
        }
    } else {
        panic!();
    }

    if let Some(def) = class.def_opt.borrow().as_ref() {
        scope.enter_class(unsafe { &**def });
    } else {
        scope.enter();
    }

    apply_class_body(&class.body, scope);

    scope.leave();
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
