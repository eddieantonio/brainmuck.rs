mmap_jit
========

![unsafe](https://img.shields.io/badge/unsafe-100%25-blue])

A repo I made to practice my knowledge of:

 - Rust's `Drop` trait
 - Using the type system to ensure data is moved appropriately
 - How to map executable memory on macOS 11.x for Apple Silicon
   Spoilers: map with the `MAP_JIT` flag as `PROT_WRITE`, then
   call `mprotect()` with `PROT_EXEC` as a page cannot be at
   `PROT_WRITE` and `PROT_EXEC` at the same time (very sensible!).

For practical purposes, use a different crate to use `mmap` memory from
within Rust.

License
=======

Unlicensed.
