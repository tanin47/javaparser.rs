extern crate javaparser;

use javaparser::{parse, tokenize};
use std::fs;
use std::time::Instant;

fn main() {
    //    let content = fs::read_to_string("./tests/fixtures/LocalCache.java").unwrap();
    //
    //    let start = Instant::now();
    //    let tokens = tokenize::apply(&content).ok().unwrap();
    //    let result = parse::apply(&tokens);
    //    let elapsed = start.elapsed().as_nanos();
    //
    //    println!("Parsing took {:?} (succeed: {})", elapsed, result.is_ok());
}
