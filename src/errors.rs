use std::io;

/// Any error that occurs as a result of compiling the source code.
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
