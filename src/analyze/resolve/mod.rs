use analyze::definition::Root;
use analyze::resolve::scope::Scope;
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
