use analyze::resolve::scope::Scope;
use parse::tree::Statement;
use semantics::Context;

pub mod variable_declarators;

pub fn apply<'def, 'def_ref, 'scope_ref>(
    stmt: &'def_ref Statement<'def>,
    context: &mut Context<'def, 'def_ref, '_>,
) {
    match stmt {
        Statement::VariableDeclarators(v) => variable_declarators::apply(v, context),
        _ => (),
    };
}
