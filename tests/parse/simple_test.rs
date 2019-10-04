use javaparser::parse;

#[test]
fn parse_minimal() {
    let result = parse::apply(
        r#"
 /* This file
 */
package dev.lilit;

import test.sub;

// Class Test is for something
class Test {
    void method(Test t) {
      Fn t = (int a) -> { run(); };
      int a = 3;
      method(1, (x) -> 2);
      return;
    }
}
        "#,
        "test.java",
    );

    assert!(result.is_ok(), format!("{:#?}", result));
}

#[test]
fn parse_minimal2() {
    let result = parse::apply(
        r#"
 /* This file
 */
package dev.lilit;

import test.sub;

// Class Test is for something
class Test {
    void method(Test t) {
      method(1);
    }
}
        "#,
        "test.java",
    );
    assert!(result.is_ok(), format!("{:#?}", result));
}
