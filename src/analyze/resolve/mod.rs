use analyze::definition::Root;
use analyze::resolve::scope::Scope;
use parse::tree::Type;
use {analyze, parse};

pub mod assign_parameterized_type;
pub mod assign_type;
pub mod grapher;
pub mod merge;
pub mod scope;

pub fn apply<'def, 'r>(units: &'r Vec<parse::tree::CompilationUnit<'def>>) -> Root<'def> {
    let mut root = merge::apply(
        units
            .iter()
            .map(|unit| analyze::build::apply(unit))
            .collect::<Vec<Root>>(),
    );

    assign_type::apply(&mut root);
    assign_parameterized_type::apply(&mut root);

    root
}

pub fn apply_type<'def, 'def_ref, 'scope_ref>(
    tpe: &'def_ref Type<'def>,
    scope: &'scope_ref Scope<'def, 'def_ref>,
) -> Type<'def> {
    if let Some(resolved) = assign_type::resolve_type(tpe, scope) {
        resolved
    } else {
        tpe.clone()
    }
}
