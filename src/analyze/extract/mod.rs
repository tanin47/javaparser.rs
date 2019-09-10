use analyze::definition::{CompilationUnit, Root};
use analyze::resolve::scope::Scope;

mod class;
mod compilation_unit;
mod package;

pub struct Position {
    pub row: usize,
    pub col: usize,
}

pub struct Location {
    pub start: Position,
    pub end: Position,
}

pub struct Usage {
    pub loc: Location,
    pub def_opt: Option<Definition>,
    pub destination: Option<Location>,
}

pub struct Extraction {
    pub usages: Vec<Usage>,
}

pub fn apply<'def>(target: &Root<'def>, root: &Root<'def>) -> Extraction {
    let mut scope = Scope {
        root,
        levels: vec![],
        specific_imports: vec![],
        wildcard_imports: vec![],
    };

    let mut extraction = Extraction { usages: vec![] };

    for unit in &root.units {
        compilation_unit::apply(unit, &mut extraction, &mut scope)
    }

    for subpackage in &root.subpackages {
        package::apply(subpackage, &mut extraction, &mut scope);
    }

    extraction
}
