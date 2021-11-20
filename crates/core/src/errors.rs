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
    pub fn new(reason: Reason, location: Location) -> Self {
        CompilationError {
            reason,
            location: Some(location),
        }
    }

    pub fn without_location(reason: Reason) -> Self {
        CompilationError {
            reason,
            location: None,
        }
    }

    pub fn location(&self) -> Option<&Location> {
        self.location.as_ref()
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

impl Location {
    pub fn new(filename: String, line_no: u32) -> Self {
        Location { filename, line_no }
    }
}

impl std::error::Error for CompilationError {}

impl fmt::Display for CompilationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let location = self
            .location
            .as_ref()
            .map(|l| format!("{}:", l))
            .unwrap_or_else(|| String::from(""));

        write!(
            f,
            "error[{:04x}]:{} {}",
            self.message_identifier(),
            location,
            self.message()
        )
    }
}

impl fmt::Display for Location {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}:{}", self.filename, self.line_no)
    }
}
