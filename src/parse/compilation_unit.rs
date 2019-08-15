use parse::combinator::{many1, opt};
use parse::def::{annotation, class, enum_def, imports, interface, modifiers, package};
use parse::tree::{CompilationUnit, CompilationUnitItem};
use parse::{ParseResult, Tokens};

pub fn parse_item(original: Tokens) -> ParseResult<CompilationUnitItem> {
    let (input, modifiers) = modifiers::parse(original)?;

    if let Ok((input, _)) = class::parse_prefix(input) {
        let (input, class) = class::parse_tail(input, modifiers)?;
        Ok((input, CompilationUnitItem::Class(class)))
    } else if let Ok((input, _)) = interface::parse_prefix(input) {
        let (input, interface) = interface::parse_tail(input, modifiers)?;
        Ok((input, CompilationUnitItem::Interface(interface)))
    } else if let Ok((input, _)) = enum_def::parse_prefix(input) {
        let (input, enum_def) = enum_def::parse_tail(input, modifiers)?;
        Ok((input, CompilationUnitItem::Enum(enum_def)))
    } else if let Ok((input, _)) = annotation::parse_prefix(input) {
        let (input, annotation) = annotation::parse_tail(input, modifiers)?;
        Ok((input, CompilationUnitItem::Annotation(annotation)))
    } else {
        Err(input)
    }
}

pub fn parse(input: Tokens) -> ParseResult<CompilationUnit> {
    let (input, package_opt) = opt(package::parse)(input)?;

    let (input, imports) = imports::parse(input)?;

    let (input, items) = many1(parse_item)(input)?;

    Ok((
        input,
        CompilationUnit {
            package_opt,
            imports,
            items,
        },
    ))
}

#[cfg(test)]
mod tests {
    use test_common::{code, span};

    use super::parse;
    use parse::tree::{
        Annotation, AnnotationBody, Class, ClassBody, CompilationUnit, CompilationUnitItem, Enum,
        Import, Interface, Package,
    };
    use parse::Tokens;

    #[test]
    fn parse_class_with_package() {
        assert_eq!(
            parse(&code(
                r#"
 /* This file
 */
package dev.lilit;

// Class Test is for something
class Test {}
interface Test2 {}
enum Test3 {}
@interface Test4 {}
        "#
            )),
            Ok((
                &[] as Tokens,
                CompilationUnit {
                    package_opt: Some(Package {
                        annotateds: vec![],
                        components: vec![span(3, 9, "dev"), span(3, 13, "lilit"),],
                    }),
                    imports: vec![],
                    items: vec![
                        CompilationUnitItem::Class(Class {
                            modifiers: vec![],
                            name: span(6, 7, "Test"),
                            type_params: vec![],
                            extend_opt: None,
                            implements: vec![],
                            body: ClassBody { items: vec![] }
                        }),
                        CompilationUnitItem::Interface(Interface {
                            modifiers: vec![],
                            name: span(7, 11, "Test2"),
                            type_params: vec![],
                            extends: vec![],
                            body: ClassBody { items: vec![] }
                        }),
                        CompilationUnitItem::Enum(Enum {
                            modifiers: vec![],
                            name: span(8, 6, "Test3"),
                            implements: vec![],
                            constants: vec![],
                            body_opt: None
                        }),
                        CompilationUnitItem::Annotation(Annotation {
                            modifiers: vec![],
                            name: span(9, 12, "Test4"),
                            body: AnnotationBody { items: vec![] }
                        }),
                    ]
                }
            ))
        );
    }

    #[test]
    fn parse_class_without_package() {
        assert_eq!(
            parse(&code(
                r#"
           class Test {}
           "#
            )),
            Ok((
                &[] as Tokens,
                CompilationUnit {
                    package_opt: None,
                    imports: vec![],
                    items: vec![CompilationUnitItem::Class(Class {
                        modifiers: vec![],
                        name: span(1, 7, "Test"),
                        type_params: vec![],
                        extend_opt: None,
                        implements: vec![],
                        body: ClassBody { items: vec![] }
                    })]
                }
            ))
        );
    }

    #[test]
    fn parse_class_with_imports() {
        assert_eq!(
            parse(&code(
                r#"
package dev.lilit;

import dev.test.*;
import dev.test;
import dev.test.Test;
import dev.test.Test.*;

class Test {}
           "#
            )),
            Ok((
                &[] as Tokens,
                CompilationUnit {
                    package_opt: Some(Package {
                        annotateds: vec![],
                        components: vec![span(1, 9, "dev"), span(1, 13, "lilit")]
                    }),
                    imports: vec![
                        Import {
                            is_static: false,
                            components: vec![span(3, 8, "dev"), span(3, 12, "test")],
                            is_wildcard: true
                        },
                        Import {
                            is_static: false,
                            components: vec![span(4, 8, "dev"), span(4, 12, "test")],
                            is_wildcard: false
                        },
                        Import {
                            is_static: false,
                            components: vec![
                                span(5, 8, "dev"),
                                span(5, 12, "test"),
                                span(5, 17, "Test")
                            ],
                            is_wildcard: false
                        },
                        Import {
                            is_static: false,
                            components: vec![
                                span(6, 8, "dev"),
                                span(6, 12, "test"),
                                span(6, 17, "Test"),
                            ],
                            is_wildcard: true
                        },
                    ],
                    items: vec![CompilationUnitItem::Class(Class {
                        modifiers: vec![],
                        name: span(8, 7, "Test"),
                        type_params: vec![],
                        extend_opt: None,
                        implements: vec![],
                        body: ClassBody { items: vec![] },
                    })]
                }
            ))
        );
    }

    #[test]
    fn parse_package_info() {
        assert_eq!(parse(&code("package dev.lilit;")), Err(&[] as Tokens,))
    }
}
