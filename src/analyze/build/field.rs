use analyze::referenceable::{Field, FieldGroup};
use parse;

pub fn build<'a>(field: &'a parse::tree::VariableDeclarator<'a>) -> Field<'a> {
    Field { name: &field.name }
}
