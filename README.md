Java parser written in Rust
----------------------------

This is a hand-written Java parser. We aim to simulate bottom-up parsing for performance reason.

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
