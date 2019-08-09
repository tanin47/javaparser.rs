use javaparser::syntax;
use javaparser::test_common::code;
use nom::error::ErrorKind;
use std::fs;

#[test]
fn all() {
    for entry in fs::read_dir("./tests/fixtures").unwrap() {
        let entry = entry.unwrap();
        println!(
            "Test: {}",
            entry.path().file_name().unwrap().to_str().unwrap()
        );
        let content = fs::read_to_string(entry.path()).unwrap();

        let result = syntax::parse(code(&content));
        assert!(result.is_ok(), format!("{:#?}", result));
    }
}
