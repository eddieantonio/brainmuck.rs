use std::collections::HashMap;
use std::fmt;
use std::io;

// Errors

/// Any error that occurs as a result of compiling the source code.
#[derive(Debug)]
pub enum CompilationError {
    IOError(io::Error),
    TooManyCloseBrackets,
}

/// An arbitrary ID assigned to a pair of [ ] branches, to associate the two.
#[derive(Debug, Clone, Hash, Copy, PartialEq, Eq)]
pub struct ConditionalID(u32);

/// A concrete offset from the beginning of a program to a specific instruction.
#[derive(Debug, Clone, Copy)]
pub struct BranchTarget(pub usize);

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

/// A representation of Brainfuck's source code that's easier to deal with than text.
/// ...at least, that would be the case in most programming languages.
pub struct AbstractSyntaxTree {
    statements: Vec<Statement>,
}

/// Representation of a Brainfuck statement in an "easier" form.
#[derive(Debug, Copy, Clone)]
pub enum Statement {
    IncrementVal,
    DecrementVal,
    IncrementAddr,
    DecrementAddr,
    PutChar,
    GetChar,
    StartConditional(ConditionalID),
    EndConditional(ConditionalID),
}

/// Parses source text (really, just a bunch of bytes) into a list of statements.
pub fn parse(source_text: &[u8]) -> Result<AbstractSyntaxTree, CompilationError> {
    use Statement::*;

    let mut statements: Vec<_> = Vec::new();
    let mut labels = ConditionalStack::new();

    for byte in source_text {
        statements.push(match byte {
            b'+' => Some(IncrementVal),
            b'-' => Some(DecrementVal),
            b'>' => Some(IncrementAddr),
            b'<' => Some(DecrementAddr),
            b'.' => Some(PutChar),
            b',' => Some(GetChar),
            b'[' => Some(StartConditional(labels.next())),
            b']' => match labels.pop() {
                Some(branch) => Some(EndConditional(branch)),
                None => {
                    return Err(CompilationError::TooManyCloseBrackets);
                }
            },
            _ => None,
        })
    }

    Ok(AbstractSyntaxTree {
        statements: statements.into_iter().flatten().collect(),
    })
}

/// Compile the AST down to bytecode.
pub fn compile_to_bytecode(ast: &AbstractSyntaxTree) -> Vec<Bytecode> {
    // Do a pre-pass to determine branch targets
    let mut conditional_branch_targets = HashMap::new();
    let mut unconditional_branch_targets = HashMap::new();
    for (ip, instr) in ast.statements.iter().enumerate() {
        match instr {
            Statement::StartConditional(branch) => {
                unconditional_branch_targets.insert(branch, BranchTarget(ip));
            }
            Statement::EndConditional(branch) => {
                conditional_branch_targets.insert(branch, BranchTarget(ip + 1));
            }
            _ => (),
        }
    }

    let mut code = Vec::new();
    for &instr in ast.statements.iter() {
        code.push(match instr {
            Statement::IncrementVal => Bytecode::ChangeVal(1),
            Statement::DecrementVal => Bytecode::ChangeVal(-1i8 as u8),
            Statement::IncrementAddr => Bytecode::ChangeAddr(1),
            Statement::DecrementAddr => Bytecode::ChangeAddr(-1),
            Statement::PutChar => Bytecode::PrintChar,
            Statement::GetChar => Bytecode::GetChar,
            Statement::StartConditional(branch) => {
                let target = *conditional_branch_targets
                    .get(&branch)
                    .expect("Branch target does not exist");
                Bytecode::BranchIfZero(target)
            }
            Statement::EndConditional(branch) => {
                let target = *unconditional_branch_targets
                    .get(&branch)
                    .expect("Branch target does not exist");
                Bytecode::BranchTo(target)
            }
        })
    }

    assert_eq!(
        code.len(),
        ast.statements.len(),
        "there should be the same number of statements as instructions"
    );

    code.push(Bytecode::Terminate);

    code
}

/// Prints Bytecode in a pseudo-assembly format.
pub fn disassemble(code: &[Bytecode]) {
    for (i, instr) in code.iter().enumerate() {
        println!("{:4}: {}", i, instr);
    }
}

// Internal data:

struct ConditionalStack {
    stack: Vec<ConditionalID>,
    next_id: u32,
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

impl ConditionalStack {
    fn new() -> Self {
        Self {
            stack: Vec::new(),
            next_id: 0,
        }
    }

    pub fn next(&mut self) -> ConditionalID {
        let current_branch = ConditionalID(self.next_id);
        self.next_id += 1;
        self.stack.push(current_branch);

        current_branch
    }

    pub fn pop(&mut self) -> Option<ConditionalID> {
        self.stack.pop()
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

impl From<io::Error> for CompilationError {
    fn from(err: io::Error) -> CompilationError {
        CompilationError::IOError(err)
    }
}
