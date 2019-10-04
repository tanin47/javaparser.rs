use javaparser::{parse, tokenize};
use std::time::{Duration, Instant};
use std::{fs, thread};

#[test]
#[ignore]
fn benchmark() {
    let content = fs::read_to_string("./tests/fixtures/LocalCache.java").unwrap();

    let mut results = vec![];
    for i in 0..10 {
        let start = Instant::now();
        let result = parse::apply(&content, "test.java").unwrap();
        let elapsed = start.elapsed().as_nanos();

        println!("{}. Parsing took {:?}", i, elapsed,);
        results.push(result)
    }

    println!("size: {}", results.len());
}
