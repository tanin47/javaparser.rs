//use nom::error::ErrorKind;
//
//use javaparser::syntax;
//use javaparser::test_common::{code, span};
//
//#[test]
//fn parse_minimal() {
//    let result = syntax::parse(code(
//        r#"
// /* This file
// */
//package dev.lilit;
//
//import test.sub;
//
//// Class Test is for something
//class Test {
//    void method(Test t) {
//      Fn t = (int a) -> { run(); };
//      int a = 3;
//      method(1, (x) -> 2);
//      return;
//    }
//}
//        "#
//        .trim(),
//    ));
//
//    assert!(result.is_ok(), format!("{:#?}", result));
//}
//
//#[test]
//fn parse_minimal2() {
//    let result = syntax::parse(code(
//        r#"
// /* This file
// */
//package dev.lilit;
//
//import test.sub;
//
//// Class Test is for something
//class Test {
//    void method(Test t) {
//      method(1);
//    }
//}
//        "#
//        .trim(),
//    ));
//
//    assert!(result.is_ok(), format!("{:#?}", result));
//}
