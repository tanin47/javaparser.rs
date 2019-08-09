Java parser written in Rust
----------------------------

[![CircleCI](https://circleci.com/gh/tanin47/javaparser.rs.svg?style=svg)](https://circleci.com/gh/tanin47/javaparser.rs)

Our hand-written Java parser supports Java 8 to 12. 
You can read about why we've decided to write a parser manually [here].


Initially, [Lilit](https://lilit.dev)'s semantics engine was built with Scala/Java. 
But, quickly enough, we have reached the limit in terms of memory footprint and computation time.
We want to reduce these 2 factors by ~10x.

We've chosen Rust because:

1. Rust can operate on string slices, and we can avoid making a lot of copies of string.
2. Memory overhead on an object is low.


Develop
--------

1. `cargo run` in order to run `src/main.rs`
2. `cargo test` in order to run all tests.
3. `cargo test -- --nocapture` in order to run all tests with STDOUT
