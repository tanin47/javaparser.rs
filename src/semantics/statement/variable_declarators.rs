use analyze::resolve::scope::Scope;
use parse::tree::{VariableDeclarator, VariableDeclarators};

pub fn apply<'def, 'def_ref, 'scope_ref>(
    declarator: &'def_ref VariableDeclarators<'def>,
    scope: &'scope_ref mut Scope<'def, 'def_ref>,
) {
    //    match &declarator.tpe {
    //        Type::Class(tpe) => scope.resolve_type(tpe),
    //    }
}
