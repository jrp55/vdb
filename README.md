# VDB
First attempt at a clean-room implementation of VDB logic in [Rust](https://www.rust-lang.org/).
Eventual goal would be to get a library interface for VDBs in the DAH for which this could be a proposed implementation.

## Prerequisites
* Tested on [Rust](https://www.rust-lang.org/) 1.44.0 but it's using a very simple subset of it. I'd imagine any recent version of Rust would do.
* Uses the `cargo` build system, which is built in with Rust.

## Do
* Read code in `src/lib.rs`.
* Test code by invoking `cargo test` in this directory.
* Try hacking on it! The "rust" extension for Visual Studio Code gets you a fairly nice IDE up and running quickly.

## Todo
* Try detecting and reacting to child engine up/down events
* Better organisation of module / tighter public interface [see here](https://doc.rust-lang.org/book/ch07-00-managing-growing-projects-with-packages-crates-and-modules.html)
* Ensure code follows Rust best practices
* Formalise library interface for VDBs and implement here.

## _Caveat Utilitor_
Still very much a toy, no real design or Rust best practice followed. Not an accurate implementation of VDBs. Subject to overhauling changes. May never be worked upon again.