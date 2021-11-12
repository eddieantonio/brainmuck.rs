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

/// Why is this exact same enum as Bytecode? Because I messed up! 🙈
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
}

impl BasicBlock {
    /// Note: this MOVES instructions into the basic block 👍🏼
    pub fn new(label: BlockLabel, instructions: Vec<ThreeAddressInstruction>) -> Self {
        BasicBlock {
            block_id: label,
            instructions,
        }
    }

    pub fn instructions(&self) -> &[ThreeAddressInstruction] {
        &self.instructions
    }

    pub fn label(&self) -> BlockLabel {
        self.block_id
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
