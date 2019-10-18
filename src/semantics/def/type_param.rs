use analyze::resolve;
use semantics::Context;
use {analyze, parse};

pub fn apply<'def>(
    type_param: &mut parse::tree::TypeParam<'def>,
    context: &mut Context<'def, '_, '_>,
) {
    let def = context
        .id_hash
        .get_by_id::<analyze::definition::TypeParam>(&type_param.id)
        .unwrap();
    type_param.def_opt.replace(Some(def));

    let mut new_extends = vec![];
    for extend_def in def.extends.borrow().iter() {
        new_extends.push(extend_def.clone());
    }

    type_param.extends = new_extends;

    context
        .scope
        .add_type_param(unsafe { &**type_param.def_opt.borrow().as_ref().unwrap() });
}
