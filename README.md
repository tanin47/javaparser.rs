Java parser written in Rust
----------------------------

[![CircleCI](https://circleci.com/gh/tanin47/javaparser.rs.svg?style=svg)](https://circleci.com/gh/tanin47/javaparser.rs)

Our hand-written Java parser supports Java 8 to 12. 

Initially, [Lilit](https://lilit.dev)'s semantics engine was built with Scala/Java. 
But, quickly enough, we have reached the limit in terms of how much we can pay for the machines. The cost is proportional to memory usage. Therefore, we want to reduce memory usage by ~10x.

We've chosen Rust because:

1. Rust can operate on string slices, and we can avoid making a lot of copies of strings.
2. Memory overhead on an object is low.
3. It's a modern language, so it's more pleasant to use.

The disadvantage we've seen so far is that Rust can't model a complex tree where nodes can refer to some other nodes. We use unsafe pointers everywhere :S

Status
-------

* Parse all Java files in OpenJDK 8 and 12 successfully
* It doesn't detect `var` in Java 10 yet.


Why we need Rust nightly?
--------------------------

Because we are using `spawn_unchecked` when spawning a thread.

Develop
--------

1. `cargo +nightly run` in order to run `src/main.rs`
2. `cargo +nightly test` in order to run all tests.
3. `cargo +nightly test -- --nocapture` in order to run all tests with STDOUT
4. Profile: `cargo +nightly profiler callgrind`

We have acceptance test in `./tests/syntax/acceptance_test.rs`, which tests the parser against real-world Java files in 
`./test/fixtures/*.java`.


Benchmark test
---------------

`cargo test benchmark --release -- --nocapture --ignored`


Real-world test
----------------

Our real-world test parses all Java files under the specified directory (recursively). 

Note that it doesn't parse `package-info.java` and `module-info.java`.

1. Change the directory location in `./tests/parse/real_world_test.rs`.
2. `cargo test real_world --release -- --nocapture --ignored`
