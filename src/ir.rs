/// A basic, internal representation of the code. This is a series of basic blocks
#[derive(Debug)]
pub struct ControlFlowGraph {
    blocks: Vec<BasicBlock>,
}

/// A basic block has only one way in and exactly one way out
#[derive(Debug)]
pub struct BasicBlock {
    block_id: BlockLabel,
    // HACK: fix this!
    pub instructions: Vec<ThreeAddressInstruction>,
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

    pub fn replace_branch_target(&mut self, target: BlockLabel) {
        use ThreeAddressInstruction::BranchIfZero;

        if !matches!(self.instructions[0], BranchIfZero(_),) {
            panic!(
                "tried to replace the branch of an unexpected basic block: {:?}",
                self
            );
        }

        self.instructions[0] = BranchIfZero(target);
    }
}
