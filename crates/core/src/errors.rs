//! All errors that can be _generated_ by the compiler.
use std::fmt;

/// Any error that occurs as a result of compiling the source code.
#[derive(Debug)]
pub struct CompilationError {
    reason: Reason,
    location: Option<Location>,
}

#[derive(Debug)]
pub struct Location {
    filename: String,
    line_no: u32,
}

#[derive(Debug)]
pub enum Reason {
    TooManyCloseBrackets,
    NotEnoughCloseBrackets,
}

impl CompilationError {
    pub fn without_location(reason: Reason) -> Self {
        CompilationError {
            reason,
            location: None,
        }
    }

    pub fn message(&self) -> &'static str {
        self.reason.message()
    }

    pub fn message_identifier(&self) -> u32 {
        self.reason.message_identifier()
    }
}

impl Reason {
    pub fn message_identifier(&self) -> u32 {
        use Reason::*;
        match self {
            TooManyCloseBrackets => 0x001,
            NotEnoughCloseBrackets => 0x002,
        }
    }

    pub fn message(&self) -> &'static str {
        use Reason::*;
        match self {
            TooManyCloseBrackets => "too many ']' brackets. Check that each '[' has a matching ']'",
            NotEnoughCloseBrackets => {
                "too many '[' brackets. Check that each '[' has a matching ']'"
            }
        }
    }
}

impl std::error::Error for CompilationError {}

impl fmt::Display for CompilationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "error[{:04x}]: {}",
            self.message_identifier(),
            self.message()
        )
    }
}
