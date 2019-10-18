use analyze::resolve::scope::Scope;
use semantics::{block, Context};
use {analyze, parse};

pub fn apply<'def>(
    field_declarators: &mut parse::tree::FieldDeclarators<'def>,
    context: &mut Context<'def, '_, '_>,
) {
    for decl in &mut field_declarators.declarators {
        apply_decl(decl, context);
    }
}

pub fn apply_decl<'def>(
    decl: &mut parse::tree::FieldDeclarator<'def>,
    context: &mut Context<'def, '_, '_>,
) {
    decl.def_opt.replace(Some(
        context
            .id_hash
            .get_by_id::<analyze::definition::FieldDef>(&decl.id)
            .unwrap(),
    ));
}
