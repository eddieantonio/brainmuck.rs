extern crate errno;

use errno::Errno;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone)]
pub enum Error {
    Internal(Errno),
    #[warn(deprecated)]
    StaticString(&'static str),
}

impl From<Errno> for Error {
    fn from(e: Errno) -> Self {
        Error::Internal(e)
    }
}

impl From<&'static str> for Error {
    fn from(s: &'static str) -> Self {
        Error::StaticString(s)
    }
}
