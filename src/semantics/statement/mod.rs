use analyze::resolve::scope::Scope;
use parse::tree::Statement;
use semantics::{expr, Context};

pub mod variable_declarators;

pub fn apply<'def, 'def_ref, 'scope_ref>(
    stmt: &'def_ref Statement<'def>,
    context: &mut Context<'def, 'def_ref, '_>,
) {
    match stmt {
        Statement::VariableDeclarators(v) => variable_declarators::apply(v, context),
        Statement::Expr(e) => expr::apply(e, context),
        _ => (),
    };
}
