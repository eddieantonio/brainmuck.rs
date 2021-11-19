//! Brainmuck internals.
//!
//! Yes, this is **massively** over-engineered, and that's by design!  The reason is that I wanted
//! to write the internals of a compiler from scratch, but without having a complicated language to
//! compile. So I chose brainfuck. It is a language intentially designed to be easy to compile, and
//! yes, I could have accomplished more or less the same thing with a stack (to maintain branch
//! targets), and a `match` char within a for-loop. But I wanted to develop a compiler in the
//! following architecture:
//!
//!  - source code is parsed into an [AbstractSyntaxTree] (AST)
//!  - the AST is _lowered_ into an **i**nternal **r**epresentation, the [ir::ControlFlowGraph] (CFG)
//!  - the control flow graph is usually the form that's easiest to perform optimizations.
//!  - the optimized CFG can then be compiled into either: [Bytecode], which is then _interpreted_
//!    or; it's machine code, which is injected into the currently running process and run
//!    directly.

extern crate mmap_jit;

use crate::bytecode::InterpretedProgram;
use crate::codegen::CodeGenerator;
use crate::jit::CompiledProgram;
use crate::parsing::AbstractSyntaxTree;

pub mod bytecode;
pub mod errors;
pub mod ir;
pub mod parsing;

mod asm;
mod codegen;
mod jit;
mod optimize;
mod program;

pub use crate::bytecode::Bytecode;
pub use crate::errors::CompilationError;
pub use crate::parsing::parse;
pub use crate::program::BrainmuckProgram;

/// Compile the AST down to bytecode, that can then be interpreted.
pub fn compile_to_bytecode(ast: &AbstractSyntaxTree) -> InterpretedProgram {
    let cfg = ir::lower(&ast);
    let cfg_opt = optimize::optimize(&cfg);

    InterpretedProgram::new(&cfg_opt)
}

/// Compile the AST to native code, injected into the current process's image.
pub fn jit_compile(ast: &AbstractSyntaxTree) -> CompiledProgram {
    let cfg = ir::lower(&ast);
    let optimized_cfg = optimize::optimize(&cfg);

    let mut gen = CodeGenerator::new();
    let code = gen.compile(&optimized_cfg);

    CompiledProgram::from_binary(&code)
}
