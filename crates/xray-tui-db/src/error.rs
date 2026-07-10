
#[derive(Debug, thiserror::Error)]
pub enum DatabaseError {
    #[error("toasty error: {0}")]
    Toasty(#[from] toasty::Error),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("uuid error: {0}")]
    Uuid(#[from] uuid::Error),
    #[error("{0}")]
    Generic(String),
}

pub type Result<T, E = DatabaseError> = std::result::Result<T, E>;
