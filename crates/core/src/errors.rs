//! All errors that can be _generated_ by the compiler.
use std::fmt;
use std::io;

/// Any error that occurs as a result of compiling the source code.
#[derive(Debug)]
pub enum CompilationError {
    IOError(io::Error),
    TooManyCloseBrackets,
    TooFewCloseBrackets,
}

impl std::error::Error for CompilationError {}

impl fmt::Display for CompilationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use CompilationError::*;
        let s = match self {
            IOError(err) => err.to_string(),
            TooManyCloseBrackets => String::from("too many close brackets"),
            TooFewCloseBrackets => {
                String::from("not enough close brackets -- did you forget to close some?")
            }
        };

        write!(f, "error: {}", s)
    }
}

impl From<io::Error> for CompilationError {
    fn from(err: io::Error) -> CompilationError {
        CompilationError::IOError(err)
    }
}
