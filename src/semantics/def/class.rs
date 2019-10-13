use analyze::resolve::scope::Scope;
use parse::tree::{Class, ClassBodyItem, EnclosingType};
use semantics::def::{field, method, type_param};
use semantics::Context;
use std::borrow::Borrow;
use {analyze, parse};

pub fn apply<'def, 'def_ref>(
    class: &'def_ref mut parse::tree::Class<'def>,
    context: &mut Context<'def, 'def_ref, '_>,
) {
    class.def_opt.replace(Some(
        context
            .id_hash
            .get_by_id::<analyze::definition::Class>(&class.id)
            .unwrap(),
    ));

    context.scope.enter();

    for t in &mut class.type_params {
        type_param::apply(t, context);
    }

    if let Some(def) = class.def_opt.borrow().as_ref() {
        context.scope.enter_class(unsafe { &**def });
    } else {
        context.scope.enter();
    }

    apply_class_body(&mut class.body, context);

    context.scope.leave();
    context.scope.leave();
}

fn apply_class_body<'def, 'def_ref, 'scope_ref>(
    body: &'def_ref mut parse::tree::ClassBody<'def>,
    context: &mut Context<'def, 'def_ref, '_>,
) {
    for item in &mut body.items {
        match item {
            ClassBodyItem::Method(m) => method::apply(m, context),
            ClassBodyItem::FieldDeclarators(f) => field::apply(f, context),
            ClassBodyItem::Class(c) => apply(c, context),
            _ => (),
        };
    }
}
