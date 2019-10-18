use analyze::resolve::scope::Scope;
use parse::tree::Name;
use semantics::Context;

pub fn apply<'def>(name: &mut Name<'def>, context: &mut Context<'def, '_, '_>) {
    name.resolved_opt
        .set(context.scope.resolve_name(name.name.fragment));
}
