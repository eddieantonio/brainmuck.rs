use crate::bytecode::compile_cfg_to_bytecode;
use crate::parsing::AbstractSyntaxTree;

pub mod bytecode;
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
