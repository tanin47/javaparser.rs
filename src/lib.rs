#![allow(unused_imports)]
#![allow(unused_variables)]
#![feature(thread_spawn_unchecked)]
#![feature(maybe_uninit_extra)]

extern crate core;
extern crate crossbeam_queue;
extern crate either;
extern crate num_cpus;
extern crate proc_macro;

use parse::tree::CompilationUnit;

#[cfg(test)]
#[macro_use]
pub mod test_common;

pub mod analyze;
pub mod extract;
pub mod parse;
pub mod semantics;
pub mod tokenize;

#[derive(Debug, PartialEq)]
pub struct JavaFile<'def> {
    pub unit: CompilationUnit<'def>,
    pub content: String,
    pub path: String,
}
unsafe impl<'a> Sync for JavaFile<'a> {}
