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
    use test_common::{generate_tokens, span};

    use super::parse;
    use parse::tree::{
        Annotation, AnnotationBody, Class, ClassBody, CompilationUnit, CompilationUnitItem, Enum,
        Import, ImportPrefix, Interface, Package,
    };
    use parse::Tokens;
    use std::cell::RefCell;

    #[test]
    fn parse_class_with_package() {
        assert_eq!(
            parse(&generate_tokens(
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
                        prefix_opt: Some(Box::new(Package {
                            prefix_opt: None,
                            annotateds: vec![],
                            name: span(3, 9, "dev"),
                            def_opt: None
                        })),
                        annotateds: vec![],
                        name: span(3, 13, "lilit"),
                        def_opt: None
                    }),
                    imports: vec![],
                    items: vec![
                        CompilationUnitItem::Class(Class {
                            modifiers: vec![],
                            name: span(6, 7, "Test"),
                            type_params: vec![],
                            extend_opt: None,
                            implements: vec![],
                            body: ClassBody { items: vec![] },
                            def_opt: RefCell::new(None),
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
            parse(&generate_tokens(
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
                        body: ClassBody { items: vec![] },
                        def_opt: RefCell::new(None)
                    })]
                }
            ))
        );
    }

    #[test]
    fn parse_class_with_imports() {
        assert_eq!(
            parse(&generate_tokens(
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
                        prefix_opt: Some(Box::new(Package {
                            prefix_opt: None,
                            annotateds: vec![],
                            name: span(1, 9, "dev"),
                            def_opt: None
                        })),
                        annotateds: vec![],
                        name: span(1, 13, "lilit"),
                        def_opt: None
                    }),
                    imports: vec![
                        Import {
                            prefix_opt: Some(Box::new(ImportPrefix {
                                prefix_opt: None,
                                name: span(3, 8, "dev"),
                                def_opt: RefCell::new(None)
                            })),
                            is_static: false,
                            is_wildcard: true,
                            name: span(3, 12, "test"),
                            def_opt: RefCell::new(None)
                        },
                        Import {
                            prefix_opt: Some(Box::new(ImportPrefix {
                                prefix_opt: None,
                                name: span(4, 8, "dev"),
                                def_opt: RefCell::new(None)
                            })),
                            is_static: false,
                            is_wildcard: false,
                            name: span(4, 12, "test"),
                            def_opt: RefCell::new(None)
                        },
                        Import {
                            prefix_opt: Some(Box::new(ImportPrefix {
                                prefix_opt: Some(Box::new(ImportPrefix {
                                    prefix_opt: None,
                                    name: span(5, 8, "dev"),
                                    def_opt: RefCell::new(None)
                                })),
                                name: span(5, 12, "test"),
                                def_opt: RefCell::new(None)
                            })),
                            is_static: false,
                            is_wildcard: false,
                            name: span(5, 17, "Test"),
                            def_opt: RefCell::new(None)
                        },
                        Import {
                            prefix_opt: Some(Box::new(ImportPrefix {
                                prefix_opt: Some(Box::new(ImportPrefix {
                                    prefix_opt: None,
                                    name: span(6, 8, "dev"),
                                    def_opt: RefCell::new(None)
                                })),
                                name: span(6, 12, "test"),
                                def_opt: RefCell::new(None)
                            })),
                            is_static: false,
                            is_wildcard: true,
                            name: span(6, 17, "Test"),
                            def_opt: RefCell::new(None)
                        },
                    ],
                    items: vec![CompilationUnitItem::Class(Class {
                        modifiers: vec![],
                        name: span(8, 7, "Test"),
                        type_params: vec![],
                        extend_opt: None,
                        implements: vec![],
                        body: ClassBody { items: vec![] },
                        def_opt: RefCell::new(None)
                    })]
                }
            ))
        );
    }

    #[test]
    fn parse_package_info() {
        assert_eq!(
            parse(&generate_tokens("package dev.lilit;")),
            Err(&[] as Tokens,)
        )
    }
}
