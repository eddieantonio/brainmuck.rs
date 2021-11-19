brainmuck.rs
============

A "optimizing" Brainfuck "JIT" compiler for Apple Silicon (AArch64).

Why?
----

I am currently learning Rust and wanted to also brush up on compiler
construction techniques, so I'm using this project as practice.

Build
-----

    cargo build

Usage
-----

    brainmuck [--no-jit] PROGRAM-NAME
    brainmuck --help
    brainmuck --version

### Options

 - `--no-jit`  uses an interpreter instead of compiling the program to machine code

License
-------

© 2021 Eddie Antonio Santos. MIT Licensed.
