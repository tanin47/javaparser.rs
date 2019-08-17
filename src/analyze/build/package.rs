use analyze::referenceable::{ClassLike, Package};
use parse;
use tokenize::span::Span;

pub fn build<'a>(
    package: &'a parse::tree::Package<'a>,
    classes: Vec<ClassLike<'a>>,
) -> Package<'a> {
    build_nested(&package.components, classes)
}

fn build_nested<'a>(components: &'a [Span], classes: Vec<ClassLike<'a>>) -> Package<'a> {
    if components.len() == 1 {
        Package {
            name: &components[0],
            subpackages: vec![],
            classes,
        }
    } else {
        Package {
            name: &components[0],
            subpackages: vec![build_nested(&components[1..], classes)],
            classes: vec![],
        }
    }
}
