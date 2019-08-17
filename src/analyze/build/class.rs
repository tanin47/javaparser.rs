use analyze::referenceable::{Class, ClassLike};
use parse;

pub fn build<'a>(class: &'a parse::tree::Class<'a>) -> ClassLike<'a> {
    ClassLike::Class(Class { name: &class.name })
}
