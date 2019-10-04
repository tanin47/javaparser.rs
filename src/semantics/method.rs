use analyze::resolve::scope::Scope;
use parse;
use semantics::block;

pub fn apply<'def, 'def_ref, 'scope_ref>(
    method: &'def_ref parse::tree::Method<'def>,
    scope: &'scope_ref mut Scope<'def, 'def_ref>,
) {
    scope.enter();

    if let Some(blk) = &method.block_opt {
        block::apply(blk, scope);
    }

    scope.leave();
}
