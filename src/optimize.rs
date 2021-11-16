//! Optimize a [ControlFlowGraph].

use crate::ir::{BasicBlock, ControlFlowGraph, ThreeAddressInstruction};

/// Perform all of the optimizations I bothered implementing.
pub fn optimize(cfg: &ControlFlowGraph) -> ControlFlowGraph {
    let blocks = cfg
        .blocks()
        .iter()
        .map(|block| BasicBlock::new(block.label(), peephole_optimize(block.instructions())))
        .collect();

    ControlFlowGraph::new(blocks)
}

/// Performs optimizations within a basic block.
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

// Makes it easier to get and replace the last element of a vector.
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
