use javaparser::parse;
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
        let file = parse::apply(&content, entry.path().to_str().unwrap());
        println!(" ({:?})", start.elapsed());
        assert!(file.is_ok(), {
            let remainder = file.err().unwrap();
            format!(
                "Parsed {} failed at line {} and column {}",
                entry.path().file_name().unwrap().to_str().unwrap(),
                remainder.line,
                remainder.col,
            )
        });
    }
}
