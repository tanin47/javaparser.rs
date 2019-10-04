use analyze::resolve::scope::Scope;
use parse::tree::Block;
use semantics::statement;

pub fn apply<'def, 'def_ref, 'scope_ref>(
    block: &'def_ref Block<'def>,
    scope: &'scope_ref mut Scope<'def, 'def_ref>,
) {
    scope.enter();
    for stmt in &block.stmts {
        statement::apply(stmt, scope);
    }
    scope.leave();
}
