use std::collections::HashMap;

mod bytecode;
mod errors;
mod ir;
mod optimize;
mod parsing;

use crate::bytecode::compile_cfg_to_bytecode;
pub use crate::bytecode::{disassemble, Bytecode};
pub use crate::errors::CompilationError;
use crate::ir::{BasicBlock, BlockLabel, ControlFlowGraph, ThreeAddressInstruction};
use crate::optimize::optimize;
pub use crate::parsing::{parse, AbstractSyntaxTree, ConditionalID, Statement};

/// Compile the AST down to bytecode.
pub fn compile_to_bytecode(ast: &AbstractSyntaxTree) -> Vec<Bytecode> {
    let cfg = lower(&ast);
    let cfg = optimize(&cfg);

    compile_cfg_to_bytecode(&cfg)
}

pub fn lower(ast: &AbstractSyntaxTree) -> ControlFlowGraph {
    use ThreeAddressInstruction::*;

    let mut blocks: Vec<BasicBlock> = Vec::new();
    let mut current_block_instrs: Vec<ThreeAddressInstruction> = Vec::new();
    let mut block_id = 0;

    let mut associated_start_block: HashMap<ConditionalID, BlockLabel> = HashMap::new();

    for &statement in ast.statements().iter() {
        match statement {
            Statement::StartConditional(cond_id) => {
                // We need to create a branch. This means a few things:

                //  1. We need to start another basic block
                //  2. ...therefore, we need to finish the current block.
                blocks.push(BasicBlock::new(BlockLabel(block_id), current_block_instrs));
                block_id += 1;

                //  3. This basic block will have exactly one instruction, to be determined later!
                let this_block_id = BlockLabel(block_id);
                blocks.push(BasicBlock::new(this_block_id, vec![NoOp]));
                //  4. We haven't seen the block that matches up with this start conditional, so
                //     we need to keep track of it for later.
                associated_start_block.insert(cond_id, this_block_id);

                block_id += 1;
                current_block_instrs = Vec::new();
            }
            Statement::EndConditional(ref cond_id) => {
                // This will always be the end of a basic block

                // We have already seen the branch target and stored it.
                let start_block = *associated_start_block
                    .get(cond_id)
                    .expect("expected to see start block already, but didn't");

                current_block_instrs.push(BranchTo(start_block));

                // This block is done...
                blocks.push(BasicBlock::new(BlockLabel(block_id), current_block_instrs));
                block_id += 1;
                current_block_instrs = Vec::new();

                // ...and we can fix the branch target to the NEXT block
                blocks[start_block.0].replace_noop_with_branch_target(BlockLabel(block_id));
            }
            _ => {
                current_block_instrs.push(statement.try_into().expect("bad statement translation"));
            }
        }
    }

    // The final block should always terminate:
    current_block_instrs.push(Terminate);

    // Finalize the last block:
    blocks.push(BasicBlock::new(BlockLabel(block_id), current_block_instrs));

    ControlFlowGraph::new(blocks)
}

pub fn print_cfg(cfg: &ControlFlowGraph) {
    use ThreeAddressInstruction::*;
    for block in cfg.blocks().iter() {
        let BlockLabel(n) = block.label();
        println!("L{}:", n);

        for &instr in block.instructions().iter() {
            match instr {
                ChangeVal(v) => println!("\tadd\t[p], [p], #{}", v as i8),
                ChangeAddr(v) => println!("\tadd\tp, p, #{}", v),
                PutChar => println!("\tputchar"),
                GetChar => println!("\tgetchar"),
                BranchIfZero(BlockLabel(n)) => println!("\tbeq\t[p], L{}", n),
                BranchTo(BlockLabel(n)) => println!("\tb\tL{}", n),
                NoOp => println!("\tnop"),
                Terminate => println!("\tterminate"),
            }
        }
    }
}

// Internal stuff:

impl TryFrom<Statement> for ThreeAddressInstruction {
    type Error = String;

    fn try_from(statement: Statement) -> Result<Self, Self::Error> {
        use self::ThreeAddressInstruction as TAC;

        match statement {
            Statement::IncrementVal => Ok(TAC::ChangeVal(1)),
            Statement::DecrementVal => Ok(TAC::ChangeVal(-1i8 as u8)),
            Statement::IncrementAddr => Ok(TAC::ChangeAddr(1)),
            Statement::DecrementAddr => Ok(TAC::ChangeAddr(-1)),
            Statement::PutChar => Ok(TAC::PutChar),
            Statement::GetChar => Ok(TAC::GetChar),
            Statement::StartConditional(_) | Statement::EndConditional(_) => Err(format!(
                "Non-trivial conversion from {:?} to branch",
                statement
            )),
        }
    }
}
