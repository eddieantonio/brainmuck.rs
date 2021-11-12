extern crate brainmuck;

use brainmuck::{CompilationError, ThreeAddressCode};
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
    let program = brainmuck::lower(&program);

    interpret(&program);
    Ok(())
}

const SIZE_OF_UNIVERSE: usize = 4096;

fn interpret(program: &[ThreeAddressCode]) {
    use ThreeAddressCode::*;

    let mut universe = [0u8; SIZE_OF_UNIVERSE];
    let mut current_address = 0;
    let mut program_counter = 0;

    while program_counter < program.len() {
        program_counter = match program[program_counter] {
            NoOp => program_counter + 1,
            ChangeVal(val) => {
                universe[current_address] = val.wrapping_add(universe[current_address]);

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
                let c = universe[current_address] as char;
                print!("{}", c);

                program_counter + 1
            }
            GetChar => {
                let mut one_byte = [0u8];
                io::stdin()
                    .read_exact(&mut one_byte)
                    .expect("could not read even a single byte!");
                universe[current_address] = one_byte[0];

                program_counter + 1
            }
            BranchIfZero(target) => {
                if universe[current_address] == 0 {
                    target.0
                } else {
                    program_counter + 1
                }
            }
            BranchTo(target) => target.0,
            Terminate => return,
        }
    }
}
