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

fn main() {
    use Instruction::*;

    let mut branch_stack: Vec<BranchID> = Vec::new();
    let mut next_branch_id = 0u32;

    let v: Vec<_> = io::stdin()
        .bytes()
        .map(|byte| match byte.expect("read from stdin") {
            b'+' => Some(IncrementVal),
            b'-' => Some(DecrementVal),
            b'>' => Some(IncrementAddr),
            b'<' => Some(DecrementAddr),
            b'.' => Some(PrintChar),
            b',' => Some(GetChar),
            b'[' => {
                let current_branch = BranchID(next_branch_id);
                next_branch_id += 1;
                branch_stack.push(current_branch);

                Some(StartBranch(current_branch))
            }
            b']' => match branch_stack.pop() {
                Some(branch) => Some(EndBranch(branch)),
                None => panic!("unbalanced branches is not implemented"),
            },
            _ => None,
        })
        .flatten()
        .collect();

    println!("{:#?}", v);
}
