use javaparser::test_common::code;
use std::fs;
use std::time::Instant;

//#[test]
//fn all() {
//    for entry in fs::read_dir("./tests/fixtures").unwrap() {
//        let entry = entry.unwrap();
//        if entry.path().is_dir() {
//            continue;
//        }
//
//        print!(
//            "Parse: {}",
//            entry.path().file_name().unwrap().to_str().unwrap()
//        );
//        let content = fs::read_to_string(entry.path()).unwrap();
//
//        let start = Instant::now();
//        let result = syntax::parse(code(&content));
//
//        assert!(
//            result.is_ok(),
//            format!(
//                "Parsed {} failed",
//                entry.path().file_name().unwrap().to_str().unwrap()
//            )
//        );
//        println!(" ({:?})", start.elapsed());
//    }
//}
//
//#[test]
//fn tokenize() {
//    for entry in fs::read_dir("./tests/fixtures").unwrap() {
//        let entry = entry.unwrap();
//        if entry.path().is_dir() {
//            continue;
//        }
//
//        print!(
//            "Tokenize: {}",
//            entry.path().file_name().unwrap().to_str().unwrap()
//        );
//        let content = fs::read_to_string(entry.path()).unwrap();
//
//        let start = Instant::now();
//        let result = tokenize::apply(&content);
//        assert!(
//            result.is_ok(),
//            format!(
//                "Tokenized {} failed",
//                entry.path().file_name().unwrap().to_str().unwrap()
//            )
//        );
//
//        println!(
//            " ({:?}, {} tokens)",
//            start.elapsed(),
//            result.ok().unwrap().len()
//        );
//    }
//}
