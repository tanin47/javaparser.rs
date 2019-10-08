use analyze::build::{field, modifier};
use analyze::definition::FieldGroup;
use parse;

pub fn build<'def, 'def_ref>(
    field_declarators: &'def_ref parse::tree::FieldDeclarators<'def>,
) -> FieldGroup<'def> {
    let mut items = vec![];

    for declarator in &field_declarators.declarators {
        items.push(field::build(declarator))
    }

    FieldGroup {
        modifiers: modifier::build(&field_declarators.modifiers),
        items,
        parse: field_declarators,
    }
}
