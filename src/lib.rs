extern crate mmap_jit;

use crate::bytecode::compile_cfg_to_bytecode;
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

pub use crate::bytecode::Bytecode;
pub use crate::errors::CompilationError;
pub use crate::parsing::parse;

/// Compile the AST down to bytecode.
pub fn compile_to_bytecode(ast: &AbstractSyntaxTree) -> Vec<Bytecode> {
    let cfg = ir::lower(&ast);
    let cfg_opt = optimize::optimize(&cfg);

    compile_cfg_to_bytecode(&cfg_opt)
}

/// Compile the AST to native code, injected into the current process's memory.
pub fn jit_compile(ast: &AbstractSyntaxTree) -> CompiledProgram {
    let cfg = ir::lower(&ast);
    let optimized_cfg = optimize::optimize(&cfg);

    let mut gen = CodeGenerator::new();
    let code = gen.compile(&optimized_cfg);

    CompiledProgram::from_binary(&code)
}
