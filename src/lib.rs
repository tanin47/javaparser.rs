#![allow(unused_imports)]
#![allow(unused_variables)]
#![feature(thread_spawn_unchecked)]

extern crate core;
extern crate crossbeam_queue;
extern crate either;
extern crate num_cpus;
extern crate proc_macro;

pub mod analyze;
pub mod extract;
pub mod parse;
pub mod semantics;
pub mod tokenize;

pub mod test_common;
