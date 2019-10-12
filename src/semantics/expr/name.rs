use analyze::resolve::scope::Scope;
use parse::tree::Name;
use semantics::Context;

pub fn apply<'def, 'def_ref, 'scope_ref>(
    name: &'def_ref Name<'def>,
    context: &mut Context<'def, 'def_ref, '_>,
) {
    name.resolved_opt
        .set(context.scope.resolve_name(name.name.fragment));
}
