//! The internal representation of a program.

use std::collections::HashMap;

use crate::parsing::{AbstractSyntaxTree, ConditionalID, Statement};

/// A basic, internal representation of the code. This is a series of basic blocks
#[derive(Debug)]
pub struct ControlFlowGraph {
    blocks: Vec<BasicBlock>,
}

/// A basic block has only one way in and exactly one way out
#[derive(Debug)]
pub struct BasicBlock {
    block_id: BlockLabel,
    instructions: Vec<ThreeAddressInstruction>,
}

/// Instructions that manipulate at most three addresses.
#[derive(Debug, Clone, Copy)]
pub enum ThreeAddressInstruction {
    ChangeVal(u8),
    ChangeAddr(i32),
    PutChar,
    GetChar,
    BranchIfZero(BlockLabel),
    BranchTo(BlockLabel),
    NoOp,
    Terminate,
}

/// A label for a basic block. Also serves as a branch target.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct BlockLabel(pub usize);

// Implementation

impl ControlFlowGraph {
    pub fn new(blocks: Vec<BasicBlock>) -> Self {
        ControlFlowGraph { blocks }
    }

    pub fn blocks(&self) -> &[BasicBlock] {
        &self.blocks
    }

    pub fn last_instruction(&self) -> Option<ThreeAddressInstruction> {
        self.blocks()
            .last()
            .and_then(|block| block.last_instruction())
    }
}

impl BasicBlock {
    /// Moves instructions into the basic block 👍🏼
    pub fn new(label: BlockLabel, instructions: Vec<ThreeAddressInstruction>) -> Self {
        BasicBlock {
            block_id: label,
            instructions,
        }
    }

    /// Return a borrowed view into all instructions in this block.
    pub fn instructions(&self) -> &[ThreeAddressInstruction] {
        &self.instructions
    }

    /// Returns this block's label.
    pub fn label(&self) -> BlockLabel {
        self.block_id
    }

    /// Returns the last instruction in this block, or None if this block is empty.
    pub fn last_instruction(&self) -> Option<ThreeAddressInstruction> {
        let n = self.instructions.len();
        self.instructions.get(n - 1).copied()
    }

    /// Replaces a basic block with a single no-op instruction to a branch with the given target.
    ///
    /// During lowering of the AST to IR, there's a case when the branch target of a conditional
    /// branch is unknown. This fixes that.
    pub fn replace_noop_with_branch_target(&mut self, target: BlockLabel) {
        use ThreeAddressInstruction::{BranchIfZero, NoOp};

        if !matches!(self.instructions[..], [NoOp]) {
            panic!(
                "tried to replace the branch of an unexpected basic block: {:?}",
                self
            );
        }

        self.instructions[0] = BranchIfZero(target);
    }
}

/// Compile an AST into a naïve control flow graph.
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

// Internal stuff:

impl TryFrom<Statement> for ThreeAddressInstruction {
    type Error = String;

    // Converts trivial AST [Statement] into a three-address address intrcution
    // Returns an [Err] when the conversion is non-trivial.
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
