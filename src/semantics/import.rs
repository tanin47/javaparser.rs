use analyze::resolve::scope::{EnclosingTypeDef, Scope};
use parse::tree::{ImportDef, ImportPrefix, ImportPrefixDef};
use {analyze, parse};

pub fn apply<'def, 'def_ref>(
    import: &parse::tree::Import<'def>,
    scope: &mut Scope<'def, 'def_ref>,
) {
    match &import.prefix_opt {
        Some(prefix) => apply_prefix(prefix, scope),
        None => (),
    };

    import
        .def_opt
        .replace(get_def(&import.prefix_opt, import.name.fragment, scope));
}

pub fn apply_prefix<'def, 'def_ref>(
    import: &parse::tree::ImportPrefix<'def>,
    scope: &mut Scope<'def, 'def_ref>,
) {
    match &import.prefix_opt {
        Some(prefix) => apply_prefix(prefix, scope),
        None => (),
    };

    import.def_opt.replace(get_prefix_def(
        &import.prefix_opt,
        import.name.fragment,
        scope,
    ));
}

fn get_def<'def, 'def_ref>(
    prefix_opt: &Option<Box<ImportPrefix<'def>>>,
    name: &'def str,
    scope: &mut Scope<'def, 'def_ref>,
) -> Option<ImportDef<'def>> {
    let result_opt = get_enclosing_type_def(prefix_opt, name, scope);

    match result_opt {
        None => None,
        Some(EnclosingTypeDef::Package(package)) => Some(ImportDef::Package(package)),
        Some(EnclosingTypeDef::Class(class)) => Some(ImportDef::Class(class)),
    }
}

fn get_prefix_def<'def, 'def_ref>(
    prefix_opt: &Option<Box<ImportPrefix<'def>>>,
    name: &'def str,
    scope: &mut Scope<'def, 'def_ref>,
) -> Option<ImportPrefixDef<'def>> {
    let result_opt = get_enclosing_type_def(prefix_opt, name, scope);

    match result_opt {
        None => None,
        Some(EnclosingTypeDef::Package(package)) => Some(ImportPrefixDef::Package(package)),
        Some(EnclosingTypeDef::Class(class)) => Some(ImportPrefixDef::Class(class)),
    }
}

fn get_enclosing_type_def<'def, 'def_ref>(
    prefix_opt: &Option<Box<ImportPrefix<'def>>>,
    name: &'def str,
    scope: &mut Scope<'def, 'def_ref>,
) -> Option<EnclosingTypeDef<'def>> {
    match prefix_opt {
        Some(prefix) => match prefix.def_opt.borrow().as_ref() {
            Some(ImportPrefixDef::Package(package)) => {
                let package = unsafe { &(**package) };
                package.find(name)
            }
            Some(ImportPrefixDef::Class(class)) => {
                let class = unsafe { &(**class) };
                class.find(name).map(|c| EnclosingTypeDef::Class(c))
            }
            None => None,
        },
        None => scope.root.find(name),
    }
}

#[cfg(test)]
mod tests {
    use analyze::test_common::{find_class, find_package, make_root, make_tokenss, make_units};
    use parse::tree::{Import, ImportDef, ImportPrefix, ImportPrefixDef};
    use std::cell::RefCell;
    use test_common::span;
    use {analyze, semantics};

    #[test]
    fn test() {
        let raws = vec![
            r#"
package dev;

import dev2.Super;
import static dev2.*;

class Test {}
        "#
            .to_owned(),
            r#"
package dev2;

class Super {}
        "#
            .to_owned(),
        ];
        let tokenss = make_tokenss(&raws);
        let units = make_units(&tokenss);
        let root = analyze::resolve::apply(&units);

        semantics::apply(units.first().unwrap(), &root);

        assert_eq!(
            units.first().unwrap().imports,
            vec![
                Import {
                    prefix_opt: Some(Box::new(ImportPrefix {
                        prefix_opt: None,
                        name: span(3, 8, "dev2"),
                        def_opt: RefCell::new(Some(ImportPrefixDef::Package(
                            root.find_package("dev2").unwrap()
                        )))
                    })),
                    is_static: false,
                    is_wildcard: false,
                    name: span(3, 13, "Super"),
                    def_opt: RefCell::new(Some(ImportDef::Class(find_class(&root, "dev2.Super"))))
                },
                Import {
                    prefix_opt: None,
                    is_static: true,
                    is_wildcard: true,
                    name: span(4, 15, "dev2"),
                    def_opt: RefCell::new(Some(ImportDef::Package(
                        root.find_package("dev2").unwrap()
                    )))
                },
            ]
        );
    }
}