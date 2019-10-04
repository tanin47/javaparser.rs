use analyze::resolve::scope::Scope;
use parse::tree::Statement;

pub mod variable_declarators;

pub fn apply<'def, 'def_ref, 'scope_ref>(
    stmt: &'def_ref Statement<'def>,
    scope: &'scope_ref mut Scope<'def, 'def_ref>,
) {
    match stmt {
        Statement::VariableDeclarators(v) => variable_declarators::apply(v, scope),
        _ => (),
    };
}
