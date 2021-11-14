extern crate mmap_jit;

use crate::bytecode::compile_cfg_to_bytecode;
use crate::parsing::AbstractSyntaxTree;

pub mod bytecode;
pub mod errors;
pub mod ir;
pub mod parsing;

mod asm;
mod codegen;
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

pub fn run_native_code(ast: &AbstractSyntaxTree) {
    let cfg = ir::lower(&ast);
    let cfg_opt = optimize::optimize(&cfg);

    codegen::run(&cfg_opt);
}
