use std::collections::HashMap;
use std::fmt;
use std::io;

mod bytecode;
mod errors;
mod ir;
mod parsing;

use crate::bytecode::compile_cfg_to_bytecode;
pub use crate::bytecode::Bytecode;
pub use crate::errors::CompilationError;
use crate::ir::{BasicBlock, BlockLabel, ControlFlowGraph, ThreeAddressInstruction};
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

fn optimize(cfg: &ControlFlowGraph) -> ControlFlowGraph {
    let blocks = cfg
        .blocks()
        .iter()
        .map(|block| BasicBlock::new(block.label(), peephole_optimize(block.instructions())))
        .collect();

    ControlFlowGraph::new(blocks)
}

fn peephole_optimize(instructions: &[ThreeAddressInstruction]) -> Vec<ThreeAddressInstruction> {
    use ThreeAddressInstruction::*;

    let mut new_instructions = vec![NoOp];

    for &instr in instructions {
        match (new_instructions.last(), instr) {
            (ChangeVal(x), ChangeVal(y)) => {
                new_instructions.replace_last(ChangeVal(x.wrapping_add(y)));
            }
            (ChangeAddr(x), ChangeAddr(y)) => new_instructions.replace_last(ChangeAddr(x + y)),
            (_, instr) => new_instructions.push(instr),
        }
    }

    new_instructions.retain(|instr| !matches!(instr, NoOp));

    new_instructions
}

/// Prints Bytecode in a pseudo-assembly format.
pub fn disassemble(code: &[Bytecode]) {
    for (i, instr) in code.iter().enumerate() {
        println!("{:4}: {}", i, instr);
    }
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

// Internal data:

trait LastNonEmptyVector<T> {
    fn last(&self) -> T;

    fn replace_last(&mut self, x: T);
}

impl<T> LastNonEmptyVector<T> for Vec<T>
where
    T: Copy,
{
    fn last(&self) -> T {
        self[self.len() - 1]
    }

    fn replace_last(&mut self, x: T) {
        let n = self.len();
        self[n - 1] = x;
    }
}

impl fmt::Display for Bytecode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Bytecode::*;
        match self {
            ChangeVal(amount) => write!(f, "[bp] <- [bp] + #{}", amount),
            ChangeAddr(amount) => write!(f, "bp <- bp + #{}", amount),
            PrintChar => write!(f, "putchar [bp]"),
            GetChar => write!(f, "getchar [bp]"),
            BranchIfZero(target) => write!(f, "beq {}", target.0),
            BranchTo(target) => write!(f, "b {}", target.0),
            NoOp => write!(f, "nop"),
            Terminate => write!(f, "ret"),
        }
    }
}

impl From<io::Error> for CompilationError {
    fn from(err: io::Error) -> CompilationError {
        CompilationError::IOError(err)
    }
}

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
