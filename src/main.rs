extern crate javaparser;
extern crate nom;

use javaparser::syntax;
use javaparser::syntax::tree::Span;
use std::time::Instant;
use std::{fs, thread, time};

//fn is_something(c: char) -> bool {
//    false
//}
//
//fn profile() {
//    let content = fs::read_to_string("./tests/fixtures/LocalCache.java").unwrap();
//    let span = Span {
//        line: 1,
//        col: 1,
//        fragment: &content,
//    };
//
//    for _ in 0..100 {
//        let start = Instant::now();
//        let result = take_till(is_something)(span) as IResult<Span, Span>;
//        let elapsed = start.elapsed().as_nanos();
//
//        let (input, c) = result.ok().unwrap();
//        println!("Nom took {:?} {}", elapsed, c.fragment.len());
//
//        let start = Instant::now();
//        let mut a: &str = "";
//        for (index, _) in content.char_indices() {
//            if ((index % 1000) == 0) {
//                a = unsafe {
//                    std::str::from_utf8_unchecked(slice::from_raw_parts(
//                        content.as_ptr(),
//                        index + 1,
//                    ))
//                };
//            }
//        }
//        let elapsed = start.elapsed().as_nanos();
//        println!("Raw took {:?} {}", elapsed, a.len());
//    }
//}

fn main() {
    let content = fs::read_to_string("./tests/fixtures/LocalCache.java").unwrap();

    let mut results = vec![];
    for i in 0..100 {
        let start = Instant::now();
        let result = syntax::parse(Span {
            line: 1,
            col: 1,
            fragment: &content,
        });

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

    thread::sleep(time::Duration::from_millis(1000000))
}
