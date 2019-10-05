use extract::Overlay;
use parse::tree::Statement;

pub mod variable_declarators;

pub fn apply<'def, 'def_ref, 'overlay_ref>(
    stmt: &'def_ref Statement<'def>,
    overlay: &'overlay_ref mut Overlay<'def>,
) {
    match stmt {
        Statement::VariableDeclarators(v) => variable_declarators::apply(v, overlay),
        Statement::Assert(_) => {}
        Statement::Block(_) => {}
        Statement::Break(_) => {}
        Statement::Class(_) => {}
        Statement::Continue(_) => {}
        Statement::Empty => {}
        Statement::DoWhile(_) => {}
        Statement::Expr(_) => {}
        Statement::ForLoop(_) => {}
        Statement::Foreach(_) => {}
        Statement::IfElse(_) => {}
        Statement::Labeled(_) => {}
        Statement::Return(_) => {}
        Statement::Switch(_) => {}
        Statement::Synchronized(_) => {}
        Statement::Throw(_) => {}
        Statement::Try(_) => {}
        Statement::WhileLoop(_) => {}
    }
}
