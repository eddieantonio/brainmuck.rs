use std::io::{self, Read};

#[derive(Debug, Clone, Copy)]
struct BranchID(u32);

#[derive(Debug, Clone, Copy)]
enum Instruction {
    IncrementVal,
    DecrementVal,
    IncrementAddr,
    DecrementAddr,
    PrintChar,
    GetChar,
    StartBranch(BranchID),
    EndBranch(BranchID),
}

fn main() -> io::Result<()> {
    let mut source_text: Vec<u8> = Vec::new();
    io::stdin().read_to_end(&mut source_text)?;
    let v = parse(&source_text)?;

    println!("{:#?}", v);

    Ok(())
}

fn parse(source_text: &[u8]) -> Result<Vec<Instruction>, io::Error> {
    use Instruction::*;

    let mut branches = BranchStack::new();

    Ok(source_text
        .iter()
        .map(|byte| match byte {
            b'+' => Some(IncrementVal),
            b'-' => Some(DecrementVal),
            b'>' => Some(IncrementAddr),
            b'<' => Some(DecrementAddr),
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
