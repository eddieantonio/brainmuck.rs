use errno::Errno;

pub type Result<T> = std::result::Result<T, MappingError>;

/// Any error thrown while mapping memory.
#[derive(Debug, Clone)]
pub enum MappingError {
    Internal(Errno),
}

impl From<Errno> for MappingError {
    fn from(e: Errno) -> Self {
        MappingError::Internal(e)
    }
}
