use std::env;
use std::fs;
use std::io::{self, Read};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct BranchID(u32);

#[derive(Debug, Clone, Copy)]
enum Instruction {
    ChangeVal(i32),
    ChangeAddr(i32),
    PrintChar,
    GetChar,
    StartBranch(BranchID),
    EndBranch(BranchID),
    NoOp,
}

#[derive(Debug)]
enum CompilationError {
    IOError(io::Error),
    TooManyCloseBrackets,
}

fn main() -> Result<(), CompilationError> {
    let args: Vec<_> = env::args().collect();

    if args.len() != 2 {
        println!("usage error: need exactly one argument");
        return Ok(());
    }

    let source_text = fs::read(&args[1])?;
    let program = parse(&source_text)?;
    let program = optimize(&program);

    interpret(&program);
    Ok(())
}

impl From<io::Error> for CompilationError {
    fn from(err: io::Error) -> CompilationError {
        CompilationError::IOError(err)
    }
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

fn parse(source_text: &[u8]) -> Result<Vec<Instruction>, CompilationError> {
    use Instruction::*;

    let mut branches = BranchStack::new();

    let mut HACK_too_many_branches = false;

    let program: Vec<_> = source_text
        .iter()
        .map(|byte| match byte {
            b'+' => Some(ChangeVal(1)),
            b'-' => Some(ChangeVal(-1)),
            b'>' => Some(ChangeAddr(1)),
            b'<' => Some(ChangeAddr(-1)),
            b'.' => Some(PrintChar),
            b',' => Some(GetChar),
            b'[' => Some(StartBranch(branches.next())),
            b']' => match branches.pop() {
                Some(branch) => Some(EndBranch(branch)),
                None => {
                    HACK_too_many_branches = true;
                    None
                }
            },
            _ => None,
        })
        .flatten()
        .collect();

    if HACK_too_many_branches {
        Err(CompilationError::TooManyCloseBrackets)
    } else {
        Ok(program)
    }
}

fn coalesce(instructions: &[Instruction]) -> Vec<Instruction> {
    use Instruction::*;

    let mut result = vec![NoOp];

    for &instr in instructions {
        match (result.last(), instr) {
            (ChangeVal(x), ChangeVal(y)) => result.replace_last(ChangeVal(x + y)),
            (ChangeAddr(x), ChangeAddr(y)) => result.replace_last(ChangeAddr(x + y)),
            _ => result.push(instr),
        }
    }

    result
}

fn remove_noop(v: &mut Vec<Instruction>) {
    v.retain(|instr| !matches!(instr, Instruction::NoOp));
}

fn optimize(v: &[Instruction]) -> Vec<Instruction> {
    let mut program = coalesce(&v);
    remove_noop(&mut program);

    program
}

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

struct BranchStack {
    stack: Vec<BranchID>,
    next_id: u32,
}

impl BranchStack {
    fn new() -> Self {
        Self {
            stack: Vec::new(),
            next_id: 0,
        }
    }

    pub fn next(&mut self) -> BranchID {
        let current_branch = BranchID(self.next_id);
        self.next_id += 1;
        self.stack.push(current_branch);

        current_branch
    }

    pub fn pop(&mut self) -> Option<BranchID> {
        self.stack.pop()
    }
}
