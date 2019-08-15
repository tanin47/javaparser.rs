Java parser written in Rust
----------------------------

[![CircleCI](https://circleci.com/gh/tanin47/javaparser.rs.svg?style=svg)](https://circleci.com/gh/tanin47/javaparser.rs)

Our hand-written Java parser supports Java 8 to 12. 

Initially, [Lilit](https://lilit.dev)'s semantics engine was built with Scala/Java. 
But, quickly enough, we have reached the limit in terms of how much we can pay for the machines. The cost is proportional to memory usage. Therefore, we want to reduce memory usage by ~10x.

We've chosen Rust because:

1. Rust can operate on string slices, and we can avoid making a lot of copies of strings.
2. Memory overhead on an object is low.
3. It's a modern language which is more pleasant to use than an ancient language.

Status
-------

* Parse all Java files in OpenJDK8 successfully


Develop
--------

1. `cargo run` in order to run `src/main.rs`
2. `cargo test` in order to run all tests.
3. `cargo test -- --nocapture` in order to run all tests with STDOUT
4. Profile: `cargo profiler callgrind`

We have acceptance test in `./tests/syntax/acceptance_test.rs`, which tests the parser against real-world Java files in 
`./test/fixtures/*.java`.


Benchmark test
---------------

`cargo test benchmark --release -- --nocapture --ignored`. Parsing `./tests/fixtures/LocalCache.java` takes ~9ms. [Javaparser](https://github.com/javaparser/javaparser) written in Java takes ~60ms.


Real-world test
----------------

Our real-world test parses all Java files under the specified directory (recursively). Note that it doesn't parse `package-info.java`.

1. Change the directory location in `./tests/parse/real_world_test.rs`.
2. `cargo test real_world --release -- --nocapture --ignored`
