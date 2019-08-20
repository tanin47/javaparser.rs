use analyze::build::{field, modifier};
use analyze::definition::FieldGroup;
use parse;

pub fn build<'a>(field_declarators: &'a parse::tree::FieldDeclarators<'a>) -> FieldGroup<'a> {
    let mut items = vec![];

    for declarator in &field_declarators.declarators {
        items.push(field::build(declarator))
    }

    FieldGroup {
        modifiers: modifier::build(&field_declarators.modifiers),
        items,
    }
}
