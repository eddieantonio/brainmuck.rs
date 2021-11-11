use std::collections::HashMap;
use std::fmt;
use std::io;

#[derive(Debug, Clone, Hash, Copy, PartialEq, Eq)]
pub struct BranchID(u32);

#[derive(Debug, Clone, Copy)]
pub enum Instruction {
    ChangeVal(i32),
    ChangeAddr(i32),
    PrintChar,
    GetChar,
    StartBranch(BranchID),
    EndBranch(BranchID),
    NoOp,
}

#[derive(Debug, Clone, Copy)]
pub struct BranchTarget(usize);

impl From<BranchTarget> for usize {
    fn from(branch: BranchTarget) -> Self {
        branch.0
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ThreeAddressCode {
    ChangeVal(u8),
    ChangeAddr(i32),
    PrintChar,
    GetChar,
    BranchIfZero(BranchTarget),
    BranchTo(BranchTarget),
    NoOp,
    Terminate,
}

#[derive(Debug)]
pub enum CompilationError {
    IOError(io::Error),
    TooManyCloseBrackets,
}

impl From<io::Error> for CompilationError {
    fn from(err: io::Error) -> CompilationError {
        CompilationError::IOError(err)
    }
}

pub fn parse(source_text: &[u8]) -> Result<Vec<Instruction>, CompilationError> {
    use Instruction::*;

    let mut program: Vec<_> = Vec::new();
    let mut branches = BranchStack::new();

    for byte in source_text {
        program.push(match byte {
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
                    return Err(CompilationError::TooManyCloseBrackets);
                }
            },
            _ => None,
        })
    }

    Ok(program.into_iter().flatten().collect())
}

pub fn optimize(v: &[Instruction]) -> Vec<Instruction> {
    let mut program = coalesce(&v);
    remove_noop(&mut program);

    program
}

pub fn lower(instructions: &[Instruction]) -> Vec<ThreeAddressCode> {
    let mut conditional_branch_targets = HashMap::new();
    let mut unconditional_branch_targets = HashMap::new();

    for (ip, instr) in instructions.iter().enumerate() {
        match instr {
            Instruction::StartBranch(branch) => {
                unconditional_branch_targets.insert(branch, BranchTarget(ip));
            }
            Instruction::EndBranch(branch) => {
                conditional_branch_targets.insert(branch, BranchTarget(ip + 1));
            }
            _ => (),
        }
    }

    let mut tac = Vec::new();
    for &instr in instructions {
        tac.push(match instr {
            Instruction::ChangeVal(val) => ThreeAddressCode::ChangeVal((val & 0xFF) as u8),
            Instruction::ChangeAddr(incr) => ThreeAddressCode::ChangeAddr(incr as i32),
            Instruction::PrintChar => ThreeAddressCode::PrintChar,
            Instruction::GetChar => ThreeAddressCode::GetChar,
            Instruction::StartBranch(branch) => {
                let target = *conditional_branch_targets
                    .get(&branch)
                    .expect("Branch target does not exist");
                ThreeAddressCode::BranchIfZero(target)
            }
            Instruction::EndBranch(branch) => {
                let target = *unconditional_branch_targets
                    .get(&branch)
                    .expect("Branch target does not exist");
                ThreeAddressCode::BranchTo(target)
            }
            Instruction::NoOp => ThreeAddressCode::NoOp,
        })
    }

    tac.push(ThreeAddressCode::Terminate);

    tac
}

pub fn disassemble(code: &[ThreeAddressCode]) {
    for (i, instr) in code.iter().enumerate() {
        println!("{:4}: {}", i, instr);
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

impl fmt::Display for ThreeAddressCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use ThreeAddressCode::*;
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
