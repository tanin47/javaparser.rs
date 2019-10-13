use analyze::resolve::scope::Scope;
use semantics::def::type_param;
use semantics::{block, Context};
use {analyze, parse};

pub fn apply<'def, 'def_ref>(
    method: &'def_ref mut parse::tree::Method<'def>,
    context: &mut Context<'def, 'def_ref, '_>,
) {
    method.def_opt.replace(Some(
        context
            .id_hash
            .get_by_id::<analyze::definition::MethodDef>(&method.id)
            .unwrap(),
    ));

    context.scope.enter();

    for t in &mut method.type_params {
        type_param::apply(t, context);
    }

    if let Some(blk) = &method.block_opt {
        block::apply(blk, context);
    }

    context.scope.leave();
}
