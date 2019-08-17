use javaparser::{parse, tokenize};
use std::time::{Duration, Instant};
use std::{fs, thread};

#[test]
#[ignore]
fn benchmark() {
    let content = fs::read_to_string("./tests/fixtures/LocalCache.java").unwrap();
    let tokens = tokenize::apply(&content).ok().unwrap();

    let mut results = vec![];
    for i in 0..10 {
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

    thread::sleep(Duration::from_millis(10000000));
}
