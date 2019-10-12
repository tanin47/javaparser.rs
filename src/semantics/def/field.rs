use analyze::resolve::scope::Scope;
use semantics::{block, Context};
use {analyze, parse};

pub fn apply<'def, 'def_ref>(
    field_declarators: &'def_ref parse::tree::FieldDeclarators<'def>,
    context: &mut Context<'def, 'def_ref, '_>,
) {
    for decl in &field_declarators.declarators {
        apply_decl(decl, context);
    }
}

pub fn apply_decl<'def, 'def_ref>(
    decl: &'def_ref parse::tree::FieldDeclarator<'def>,
    context: &mut Context<'def, 'def_ref, '_>,
) {
    decl.def_opt.replace(Some(
        context
            .id_hash
            .get_by_id::<analyze::definition::FieldDef>(&decl.id)
            .unwrap(),
    ));
}
