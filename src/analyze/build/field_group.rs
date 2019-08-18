use analyze::build::field;
use analyze::referenceable::FieldGroup;
use parse;

pub fn build<'a>(field_declarators: &'a parse::tree::FieldDeclarators<'a>) -> FieldGroup<'a> {
    let mut items = vec![];

    for declarator in &field_declarators.declarators {
        items.push(field::build(declarator))
    }

    FieldGroup { items }
}
