use analyze::resolve::scope::Scope;
use parse::tree::Block;
use semantics::{statement, Context};

pub fn apply<'def, 'def_ref>(
    block: &'def_ref Block<'def>,
    context: &mut Context<'def, 'def_ref, '_>,
) {
    context.scope.enter();
    for stmt in &block.stmts {
        statement::apply(stmt, context);
    }
    context.scope.leave();
}
