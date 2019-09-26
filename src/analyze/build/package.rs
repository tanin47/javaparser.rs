use analyze::build::scope::Scope;
use analyze::build::{class, compilation_unit};
use analyze::definition::{Class, Package};
use parse;
use parse::tree::CompilationUnitItem;
use tokenize::span::Span;

pub fn build<'def, 'scope_ref, 'def_ref>(
    package: &'def_ref parse::tree::Package<'def>,
    unit: &'def_ref parse::tree::CompilationUnit<'def>,
    scope: &'scope_ref mut Scope,
) -> Package<'def>
where
    'def: 'scope_ref,
{
    build_nested(&package.components, unit, scope)
}

fn build_nested<'def, 'b>(
    components: &'a [Span],
    unit: &'a parse::tree::CompilationUnit<'a>,
    scope: &'b mut Scope,
) -> Package<'a>
where
    'a: 'b,
{
    scope.wrap(components[0].fragment, |scope| {
        if components.len() == 1 {
            let unit = compilation_unit::build_unit(unit, scope);
            Package {
                import_path: scope.get_import_path(),
                name: components[0].fragment.to_owned(),
                subpackages: vec![],
                units: vec![unit],
            }
        } else {
            Package {
                import_path: scope.get_import_path(),
                name: components[0].fragment.to_owned(),
                subpackages: vec![build_nested(&components[1..], unit, scope)],
                units: vec![],
            }
        }
    })
}
