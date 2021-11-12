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

/// A label for a basic block. Also serves as a branch target.
#[derive(Debug, Clone, Copy)]
pub struct BlockLabel(pub usize);

/// A basic, internal representation of the code. This is a series of basic blocks
#[derive(Debug)]
pub struct ControlFlowGraph {
    blocks: Vec<BasicBlock>,
}

/// A basic block has only one way in and exactly one way out
#[derive(Debug)]
pub struct BasicBlock {
    block_id: BlockLabel,
    instructions: Vec<ThreeAddressInstruction>,
}

/// Why is this exact same enum as Bytecode? Because I messed up! 🙈
#[derive(Debug, Clone, Copy)]
pub enum ThreeAddressInstruction {
    ChangeVal(u8),
    ChangeAddr(i32),
    PutChar,
    GetChar,
    BranchIfZero(BlockLabel),
    BranchTo(BlockLabel),
    NoOp,
    Terminate,
}

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
    let cfg = lower(&ast);
    let cfg = optimize(&cfg);
    print_cfg(&cfg);

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

pub fn lower(ast: &AbstractSyntaxTree) -> ControlFlowGraph {
    use ThreeAddressInstruction::*;

    let mut blocks: Vec<BasicBlock> = Vec::new();
    let mut current_block_instrs: Vec<ThreeAddressInstruction> = Vec::new();
    let mut block_id = 0;

    let mut associated_start_block: HashMap<ConditionalID, BlockLabel> = HashMap::new();

    for &statement in ast.statements.iter() {
        match statement {
            Statement::StartConditional(cond_id) => {
                // We need to create a branch. This means a few things:

                //  1. We need to start another basic block
                //  2. ...therefore, we need to finish the current block.
                blocks.push(BasicBlock {
                    block_id: BlockLabel(block_id),
                    instructions: current_block_instrs,
                });
                block_id += 1;

                //  3. This basic block will have exactly one instruction, to be determined later!
                let this_block_id = BlockLabel(block_id);
                blocks.push(BasicBlock {
                    block_id: this_block_id,
                    instructions: vec![NoOp],
                });
                //  4. We haven't seen the block that matches up with this start conditional, so
                //     we need to keep track of it for later.
                associated_start_block.insert(cond_id, this_block_id);

                block_id += 1;
                current_block_instrs = Vec::new();
            }
            Statement::EndConditional(ref cond_id) => {
                // This will always be the end of a basic block

                // We have already seen the branch target and stored it.
                let start_block = *associated_start_block
                    .get(cond_id)
                    .expect("expected to see start block already, but didn't");

                current_block_instrs.push(BranchTo(start_block));

                // This block is done...
                blocks.push(BasicBlock {
                    block_id: BlockLabel(block_id),
                    instructions: current_block_instrs,
                });
                block_id += 1;
                current_block_instrs = Vec::new();

                // ...and we can fix the branch target to the NEXT block
                blocks[start_block.0].instructions[0] = BranchIfZero(BlockLabel(block_id));
            }
            _ => {
                current_block_instrs.push(statement.try_into().expect("bad statement translation"));
            }
        }
    }

    // The final block should always terminate:
    current_block_instrs.push(Terminate);

    // Finalize the last block:
    blocks.push(BasicBlock {
        block_id: BlockLabel(block_id),
        instructions: current_block_instrs,
    });

    ControlFlowGraph { blocks }
}

fn optimize(cfg: &ControlFlowGraph) -> ControlFlowGraph {
    let blocks = cfg
        .blocks
        .iter()
        .map(|block| {
            let BasicBlock {
                block_id,
                instructions,
            } = block;

            BasicBlock {
                block_id: *block_id,
                instructions: peephole_optimize(&instructions),
            }
        })
        .collect();

    ControlFlowGraph { blocks }
}

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

/// Prints Bytecode in a pseudo-assembly format.
pub fn disassemble(code: &[Bytecode]) {
    for (i, instr) in code.iter().enumerate() {
        println!("{:4}: {}", i, instr);
    }
}

fn print_cfg(cfg: &ControlFlowGraph) {
    use ThreeAddressInstruction::*;
    for block in cfg.blocks.iter() {
        let BlockLabel(n) = block.block_id;
        println!("L{}:", n);

        for &instr in block.instructions.iter() {
            match instr {
                ChangeVal(v) => println!("\tadd\t[p], [p], #{}", v as i8),
                ChangeAddr(v) => println!("\tadd\tp, p, #{}", v),
                PutChar => println!("\tputchar"),
                GetChar => println!("\tgetchar"),
                BranchIfZero(BlockLabel(n)) => println!("\tbeq\t[p], L{}", n),
                BranchTo(BlockLabel(n)) => println!("\tb\tL{}", n),
                NoOp => println!("\tnop"),
                Terminate => println!("\tterminate"),
            }
        }
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

impl TryFrom<Statement> for ThreeAddressInstruction {
    type Error = String;

    fn try_from(statement: Statement) -> Result<Self, Self::Error> {
        use self::ThreeAddressInstruction as TAC;

        match statement {
            Statement::IncrementVal => Ok(TAC::ChangeVal(1)),
            Statement::DecrementVal => Ok(TAC::ChangeVal(-1i8 as u8)),
            Statement::IncrementAddr => Ok(TAC::ChangeAddr(1)),
            Statement::DecrementAddr => Ok(TAC::ChangeAddr(-1)),
            Statement::PutChar => Ok(TAC::PutChar),
            Statement::GetChar => Ok(TAC::GetChar),
            Statement::StartConditional(_) | Statement::EndConditional(_) => Err(format!(
                "Non-trivial conversion from {:?} to branch",
                statement
            )),
        }
    }
}
