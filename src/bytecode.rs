use std::collections::HashMap;
use std::fmt;

use crate::ir::{ControlFlowGraph, ThreeAddressInstruction};

/// "Bytecode" is a misnomer, but it's the best idea for what this is. It's pseudo-assembly and one
/// can write an intrepretter for it pretty easily 👀
#[derive(Debug, Clone, Copy)]
pub enum Bytecode {
    ChangeVal(u8),
    ChangeAddr(i32),
    PrintChar,
    GetChar,
    BranchIfZero(BranchTarget),
    BranchTo(BranchTarget),
    NoOp,
    Terminate,
}

/// A concrete offset from the beginning of a program to a specific instruction.
#[derive(Debug, Clone, Copy)]
pub struct BranchTarget(pub usize);

/// Convert a ControlFlowGraph to Bytecode.
pub fn compile_cfg_to_bytecode(cfg: &ControlFlowGraph) -> Vec<Bytecode> {
    let mut branch_targets = HashMap::new();
    let mut incomplete_instructions = Vec::new();
    let mut code = Vec::new();
    let mut pc = 0;

    // First pass. Generate code, but don't try making valid branch targets.
    for block in cfg.blocks().iter() {
        use ThreeAddressInstruction::*;

        let block_id = block.label();
        let instructions = block.instructions();
        branch_targets.insert(block_id, BranchTarget(pc));

        for &instr in instructions {
            code.push(match instr {
                ChangeVal(c) => Bytecode::ChangeVal(c),
                ChangeAddr(c) => Bytecode::ChangeAddr(c),
                PutChar => Bytecode::PrintChar,
                GetChar => Bytecode::GetChar,
                BranchIfZero(label) => {
                    incomplete_instructions.push((pc, label));
                    Bytecode::BranchIfZero(BranchTarget(0))
                }
                BranchTo(label) => {
                    incomplete_instructions.push((pc, label));
                    Bytecode::BranchTo(BranchTarget(0))
                }
                NoOp => {
                    continue;
                }
                Terminate => Bytecode::Terminate,
            });

            pc += 1;
        }
    }

    // Second pass: patch in branch targets
    for (i, ref label) in incomplete_instructions {
        use Bytecode::*;

        let instr = code[i];
        let target = *branch_targets
            .get(label)
            .expect("branch target should have been determined in the first pass");

        code[i] = match instr {
            BranchIfZero(_) => BranchIfZero(target),
            BranchTo(_) => BranchTo(target),
            _ => panic!("replacing branch not supported for {:?}", instr),
        };
    }

    code
}

/// Prints Bytecode in a pseudo-assembly format.
pub fn disassemble(code: &[Bytecode]) {
    for (i, instr) in code.iter().enumerate() {
        println!("{:4}: {}", i, instr);
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
