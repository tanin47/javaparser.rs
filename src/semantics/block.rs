use analyze::resolve::scope::Scope;
use parse::tree::Block;
use semantics::{statement, Context};

pub fn apply<'def>(block: &mut Block<'def>, context: &mut Context<'def, '_, '_>) {
    context.scope.enter();
    for stmt in &mut block.stmts {
        statement::apply(stmt, context);
    }
    context.scope.leave();
}
