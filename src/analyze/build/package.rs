use analyze::build::class;
use analyze::build::compilation_unit::build_items;
use analyze::build::scope::Scope;
use analyze::referenceable::{Class, Package};
use parse;
use parse::tree::CompilationUnitItem;
use tokenize::span::Span;

pub fn build<'a, 'b>(
    package: &'a parse::tree::Package<'a>,
    items: &'a [CompilationUnitItem<'a>],
    scope: &'b mut Scope,
) -> Package<'a>
where
    'a: 'b,
{
    build_nested(&package.components, items, scope)
}

fn build_nested<'a, 'b>(
    components: &'a [Span],
    items: &'a [CompilationUnitItem<'a>],
    scope: &'b mut Scope,
) -> Package<'a>
where
    'a: 'b,
{
    scope.wrap(components[0].fragment, |scope| {
        if components.len() == 1 {
            let (classes, interfaces) = build_items(items, scope);
            Package {
                import_path: scope.get_import_path(),
                name: &components[0],
                subpackages: vec![],
                classes,
                interfaces,
            }
        } else {
            Package {
                import_path: scope.get_import_path(),
                name: &components[0],
                subpackages: vec![build_nested(&components[1..], items, scope)],
                classes: vec![],
                interfaces: vec![],
            }
        }
    })
}
