extern crate mmap_jit;

use crate::bytecode::compile_cfg_to_bytecode;
use crate::parsing::AbstractSyntaxTree;

pub mod bytecode;
mod codegen;
pub mod errors;
pub mod ir;
mod optimize;
pub mod parsing;

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
