use std::io::{self, Read};

#[derive(Debug, Clone, Copy)]
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

fn main() -> io::Result<()> {
    let mut source_text: Vec<u8> = Vec::new();
    io::stdin().read_to_end(&mut source_text)?;
    let v = parse(&source_text)?;
    let mut opt = coalesce(&v);
    remove_noop(&mut opt);

    println!("{:#?}", opt);

    Ok(())
}

fn parse(source_text: &[u8]) -> Result<Vec<Instruction>, io::Error> {
    use Instruction::*;

    let mut branches = BranchStack::new();

    Ok(source_text
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
                None => panic!("unbalanced branches is not implemented"),
            },
            _ => None,
        })
        .flatten()
        .collect())
}

fn coalesce(instructions: &Vec<Instruction>) -> Vec<Instruction> {
    use Instruction::*;

    let mut result = vec![NoOp];

    for &instr in instructions {
        match (result.last(), instr) {
            (ChangeVal(x), ChangeVal(y)) => result.replace_last(ChangeVal(x + y)),
            (ChangeAddr(x), ChangeAddr(y)) => result.replace_last(ChangeVal(x + y)),
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
