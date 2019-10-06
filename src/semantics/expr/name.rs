use analyze::resolve::scope::Scope;
use parse::tree::Name;

pub fn apply<'def, 'def_ref, 'scope_ref>(
    name: &'def_ref Name<'def>,
    scope: &'scope_ref mut Scope<'def, 'def_ref>,
) {
    name.resolved_opt
        .set(scope.resolve_name(name.name.fragment));
}
