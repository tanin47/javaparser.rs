use javaparser::syntax;
use javaparser::test_common::code;
use nom::error::ErrorKind;
use std::fs;
use std::time::Instant;

#[test]
fn all() {
    for entry in fs::read_dir("./tests/fixtures").unwrap() {
        let entry = entry.unwrap();
        if entry.path().is_dir() {
            continue;
        }

        print!(
            "Test: {}",
            entry.path().file_name().unwrap().to_str().unwrap()
        );
        let content = fs::read_to_string(entry.path()).unwrap();

        let start = Instant::now();
        let result = syntax::parse(code(&content));
        println!(" ({:?})", start.elapsed());

        assert!(result.is_ok(), format!("{:#?}", result));
    }
}
