pub mod bytecode;
pub mod errors;
pub mod ir;
mod optimize;
pub mod parsing;

use crate::bytecode::compile_cfg_to_bytecode;
pub use crate::bytecode::{disassemble, Bytecode};
pub use crate::errors::CompilationError;
pub use crate::parsing::{parse, AbstractSyntaxTree, ConditionalID, Statement};

/// Compile the AST down to bytecode.
pub fn compile_to_bytecode(ast: &AbstractSyntaxTree) -> Vec<Bytecode> {
    let cfg = ir::lower(&ast);
    let cfg_opt = optimize::optimize(&cfg);

    compile_cfg_to_bytecode(&cfg_opt)
}
