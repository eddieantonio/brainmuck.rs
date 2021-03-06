//! "Parse" brainfuck source text.

use crate::errors::{CompilationError, Location, Reason};

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

/// An arbitrary ID assigned to a pair of [ ] branches, to associate the two.
#[derive(Debug, Clone, Hash, Copy, PartialEq, Eq)]
pub struct ConditionalID(u32);

// public functions

/// Parses source text (really, just a bunch of bytes) into a list of statements.
pub fn parse(filename: &str, source_text: &[u8]) -> Result<AbstractSyntaxTree, CompilationError> {
    use Statement::*;

    let mut statements: Vec<_> = Vec::new();
    let mut labels = ConditionalStack::new();
    let mut location = LocationTracker::new(filename);

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
                    return Err(location.into_error(Reason::TooManyCloseBrackets));
                }
            },
            b'\n' => {
                location.increment_line_number();
                None
            }
            _ => None,
        })
    }

    if labels.has_unmatched_brackets() {
        return Err(location.into_error(Reason::TooManyOpenBrackets));
    }

    Ok(AbstractSyntaxTree {
        statements: statements.into_iter().flatten().collect(),
    })
}

// Implementations
impl AbstractSyntaxTree {
    pub fn statements(&self) -> &[Statement] {
        &self.statements[..]
    }
}

// Private data structurs

struct ConditionalStack {
    stack: Vec<ConditionalID>,
    next_id: u32,
}

impl ConditionalStack {
    fn new() -> Self {
        Self {
            stack: Vec::new(),
            next_id: 0,
        }
    }

    pub fn has_unmatched_brackets(&self) -> bool {
        !self.stack.is_empty()
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

struct LocationTracker {
    line_number: u32,
    filename: String,
}

impl LocationTracker {
    fn new(filename: &str) -> Self {
        let filename = filename.to_string();
        let line_number = 1;

        LocationTracker {
            filename,
            line_number,
        }
    }

    fn increment_line_number(&mut self) {
        self.line_number += 1;
    }

    fn into_error(self, reason: Reason) -> CompilationError {
        CompilationError::new(reason, Location::new(self.filename, self.line_number))
    }
}
