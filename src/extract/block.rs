use extract::{statement, Overlay};
use parse::tree::Block;

pub fn apply<'def, 'def_ref, 'overlay_ref>(
    block: &'def_ref Block<'def>,
    overlay: &'overlay_ref mut Overlay<'def>,
) {
    for stmt in &block.stmts {
        statement::apply(stmt, overlay);
    }
}
