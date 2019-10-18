use parse::tree::{ReturnStmt, Type};
use semantics::{expr, Context};

pub fn apply<'def>(return_stmt: &mut ReturnStmt<'def>, context: &mut Context<'def, '_, '_>) {
    if let Some(e) = &mut return_stmt.expr_opt {
        expr::apply(e, &Type::UnknownType, context);
    }
}
