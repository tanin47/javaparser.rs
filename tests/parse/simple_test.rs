use javaparser::parse;
use javaparser::test_common::{code, span};

#[test]
fn parse_minimal() {
    let tokens = code(
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
    );

    let result = parse::apply(&tokens);

    assert!(result.is_ok(), format!("{:#?}", result));
}

#[test]
fn parse_minimal2() {
    let tokens = code(
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
    );
    let result = parse::apply(&tokens);

    assert!(result.is_ok(), format!("{:#?}", result));
}
