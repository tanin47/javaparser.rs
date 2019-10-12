use analyze;
use analyze::resolve::scope::Scope;
use parse;
use parse::id_gen::IdGen;
use semantics::id_hash::IdHash;
use std::collections::HashMap;

pub mod block;
pub mod class;
pub mod compilation_unit;
pub mod expr;
pub mod id_hash;
pub mod import;
pub mod method;
pub mod statement;

pub struct Context<'def, 'def_ref, 'id_hash_ref> {
    pub scope: Scope<'def, 'def_ref>,
    pub id_hash: &'id_hash_ref IdHash,
}

pub fn apply<'def>(
    target: &parse::tree::CompilationUnit<'def>,
    root: &analyze::definition::Root<'def>,
    id_hash: &IdHash,
) {
    let mut context = Context {
        scope: Scope {
            root,
            levels: vec![],
            specific_imports: vec![],
            wildcard_imports: vec![],
        },
        id_hash,
    };
    compilation_unit::apply(target, &mut context)
}
