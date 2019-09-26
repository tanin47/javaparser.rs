use analyze::resolve::scope::{EnclosingTypeDef, Scope};
use semantics::tree::{CompilationUnit, Import, ImportDef, ImportPrefix, ImportPrefixDef};
use {analyze, parse};

pub fn apply<'def, 'def_ref>(
    import: &parse::tree::Import<'def>,
    scope: &mut Scope<'def, 'def_ref>,
) -> Import<'def> {
    let mut prefix_opt: Option<ImportPrefix> = None;
    for component in &import.components[0..(import.components.len() - 1)] {
        let def_opt = get_prefix_def(&prefix_opt, component.fragment, scope);
        prefix_opt = Some(ImportPrefix {
            span: component.clone(),
            prefix_opt: prefix_opt.map(Box::new),
            def_opt,
        });
    }

    let span = import.components.last().unwrap().clone();
    let def_opt = get_def(&prefix_opt, span.fragment, scope);
    Import {
        span,
        prefix_opt,
        is_static: import.is_static,
        is_wildcard: import.is_wildcard,
        def_opt,
    }
}

fn get_def<'def, 'def_ref>(
    prefix_opt: &Option<ImportPrefix<'def>>,
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
    prefix_opt: &Option<ImportPrefix<'def>>,
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
    prefix_opt: &Option<ImportPrefix<'def>>,
    name: &'def str,
    scope: &mut Scope<'def, 'def_ref>,
) -> Option<EnclosingTypeDef<'def>> {
    match prefix_opt {
        Some(prefix) => match prefix.def_opt {
            Some(ImportPrefixDef::Package(package)) => {
                let package = unsafe { &(*package) };
                package.find(name)
            }
            Some(ImportPrefixDef::Class(class)) => {
                let class = unsafe { &(*class) };
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
    use semantics::tree::{Import, ImportDef, ImportPrefix, ImportPrefixDef};
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

        let result = semantics::apply(units.first().unwrap(), &root);

        assert_eq!(
            result.imports,
            vec![
                Import {
                    span: span(3, 13, "Super"),
                    prefix_opt: Some(ImportPrefix {
                        span: span(3, 8, "dev2"),
                        prefix_opt: None,
                        def_opt: Some(ImportPrefixDef::Package(root.find_package("dev2").unwrap()))
                    }),
                    is_static: false,
                    is_wildcard: false,
                    def_opt: Some(ImportDef::Class(find_class(&root, "dev2.Super")))
                },
                Import {
                    span: span(4, 15, "dev2"),
                    prefix_opt: None,
                    is_static: true,
                    is_wildcard: true,
                    def_opt: Some(ImportDef::Package(root.find_package("dev2").unwrap()))
                },
            ]
        );
    }
}
