use analyze::build::{class, package};
use analyze::referenceable::{ClassLike, Package, Root};
use either::Either;
use parse;

pub fn build<'a>(unit: &'a parse::tree::CompilationUnit<'a>) -> Root<'a> {
    let mut classes = vec![];

    for item in &unit.items {
        classes.push(build_item(item))
    }

    let (subpackages, classes) = match &unit.package_opt {
        Some(package) => (vec![package::build(package, classes)], vec![]),
        None => (vec![], classes),
    };
    Root {
        subpackages,
        classes,
    }
}

pub fn build_item<'a>(item: &'a parse::tree::CompilationUnitItem<'a>) -> ClassLike<'a> {
    match item {
        parse::tree::CompilationUnitItem::Class(c) => class::build(c),
        parse::tree::CompilationUnitItem::Interface(interface) => panic!(),
        parse::tree::CompilationUnitItem::Annotation(annotation) => panic!(),
        parse::tree::CompilationUnitItem::Enum(enum_def) => panic!(),
    }
}

#[cfg(test)]
mod tests {
    use super::build;
    use analyze::referenceable::{Class, ClassLike, Package, Root};
    use test_common::{code, parse, span};

    #[test]
    fn test_without_package() {
        assert_eq!(
            build(&parse(&code(
                r#"
class Test {}
        "#,
            ))),
            Root {
                subpackages: vec![],
                classes: vec![ClassLike::Class(Class {
                    name: &span(1, 7, "Test")
                })],
            }
        )
    }

    #[test]
    fn test_with_package() {
        assert_eq!(
            build(&parse(&code(
                r#"
package dev.lilit;

class Test {}
        "#,
            ))),
            Root {
                subpackages: vec![Package {
                    name: &span(1, 9, "dev"),
                    subpackages: vec![Package {
                        name: &span(1, 13, "lilit"),
                        subpackages: vec![],
                        classes: vec![ClassLike::Class(Class {
                            name: &span(3, 7, "Test")
                        })],
                    }],
                    classes: vec![]
                }],
                classes: vec![]
            }
        )
    }

}
