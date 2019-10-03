use javaparser::parse;
use javaparser::test_common::generate_tokens;
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
            "Parse: {}",
            entry.path().file_name().unwrap().to_str().unwrap()
        );
        let content = fs::read_to_string(entry.path()).unwrap();

        let start = Instant::now();
        let tokens = generate_tokens(&content);
        let result = parse::apply(&tokens);
        println!(" ({:?})", start.elapsed());
        assert!(result.is_ok(), {
            let remainder = result.err().unwrap();
            format!(
                "Parsed {} failed at line {} and column {}",
                entry.path().file_name().unwrap().to_str().unwrap(),
                remainder[0].span().line,
                remainder[0].span().col,
            )
        });
    }
}
