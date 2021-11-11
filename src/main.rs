extern crate brainmuck;

use brainmuck::{BranchID, CompilationError, Instruction};
use std::env;
use std::fs;
use std::io::{self, Read};

fn main() -> Result<(), CompilationError> {
    let args: Vec<_> = env::args().collect();

    if args.len() != 2 {
        println!("usage error: need exactly one argument");
        return Ok(());
    }

    let source_text = fs::read(&args[1])?;
    let program = brainmuck::parse(&source_text)?;
    let program = brainmuck::optimize(&program);

    interpret(&program);
    Ok(())
}

const SIZE_OF_UNIVERSE: usize = 4096;

fn interpret(program: &[Instruction]) {
    use std::num::Wrapping;
    use Instruction::*;

    let mut universe = [Wrapping(0u8); SIZE_OF_UNIVERSE];
    let mut current_address = 0;
    let mut program_counter = 0;

    while program_counter < program.len() {
        program_counter = match program[program_counter] {
            NoOp => program_counter + 1,
            ChangeVal(val) => {
                universe[current_address] += Wrapping((val & 0xFF) as u8);

                program_counter + 1
            }
            ChangeAddr(incr) => {
                let address = current_address as i32 + incr;

                if address as usize >= SIZE_OF_UNIVERSE {
                    panic!("Runtime error: address went beyond the end of the universe");
                } else if address < 0 {
                    panic!("Runtime error: address went below zero");
                } else {
                    current_address = address as usize;
                }

                program_counter + 1
            }
            PrintChar => {
                let c = universe[current_address].0 as char;
                print!("{}", c);

                program_counter + 1
            }
            GetChar => {
                let mut one_byte = [0u8];
                io::stdin()
                    .read_exact(&mut one_byte)
                    .expect("could not read even a single byte!");
                universe[current_address] = Wrapping(one_byte[0]);

                program_counter + 1
            }
            StartBranch(start) => {
                if universe[current_address].0 == 0 {
                    find_end_branch_target(start, &program, program_counter)
                } else {
                    program_counter + 1
                }
            }
            EndBranch(end) => find_start_branch_target(end, &program, program_counter),
        }
    }
}

fn find_end_branch_target(start: BranchID, program: &[Instruction], pc: usize) -> usize {
    let mut increment = 0;

    for &instr in &program[pc..] {
        match instr {
            Instruction::EndBranch(end) if start == end => break,
            _ => increment += 1,
        }
    }
    assert!(increment > 0);
    assert!(matches!(program[pc + increment], Instruction::EndBranch(_)));

    pc + increment + 1
}

fn find_start_branch_target(end: BranchID, program: &[Instruction], pc: usize) -> usize {
    let mut target = None;

    for i in (0..pc).rev() {
        match program[i] {
            Instruction::StartBranch(start) if start == end => {
                target.replace(i);
                break;
            }
            _ => continue,
        }
    }

    target.expect("Somehow did not find start of branch")
}
