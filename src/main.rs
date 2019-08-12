extern crate javaparser;

use javaparser::{parse, tokenize};
use std::fs;
use std::time::Instant;

fn main() {
    let content = fs::read_to_string("./tests/fixtures/LocalCache.java").unwrap();
    let tokens = tokenize::apply(&content).ok().unwrap();

    let mut results = vec![];
    for i in 0..100 {
        let start = Instant::now();
        let _ = tokenize::apply(&content).ok().unwrap();
        let result = parse::apply(&tokens);
        let elapsed = start.elapsed().as_nanos();

        println!(
            "{}. Parsing took {:?} (succeed: {})",
            i,
            elapsed,
            result.is_ok()
        );
        results.push(result)
    }

    println!("size: {}", results.len());
}
