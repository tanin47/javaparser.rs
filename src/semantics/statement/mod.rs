use analyze::resolve::scope::Scope;
use parse::tree::{Statement, Type};
use semantics::{expr, Context};

pub mod return_stmt;
pub mod variable_declarators;

pub fn apply<'def>(stmt: &mut Statement<'def>, context: &mut Context<'def, '_, '_>) {
    match stmt {
        Statement::VariableDeclarators(v) => variable_declarators::apply(v, context),
        Statement::Expr(e) => expr::apply(e, &Type::UnknownType, context),
        Statement::Return(r) => return_stmt::apply(r, context),
        _ => (),
    };
}
