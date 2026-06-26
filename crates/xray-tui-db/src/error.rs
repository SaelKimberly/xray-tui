use crate::models::{Profile, ProfileExtension, ServerStat};

#[derive(Debug, thiserror::Error)]
pub enum DatabaseError {
    #[error("turso error: {0}")]
    Turso(#[from] turso::Error),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("uuid error: {0}")]
    Uuid(#[from] uuid::Error),
    #[error("{0}")]
    Generic(String),
}

pub type Result<T, E = DatabaseError> = std::result::Result<T, E>;

/// Result tuple returned by `get_all_profiles_with_details`.
pub type ProfileWithDetails = (Profile, Option<ProfileExtension>, Option<ServerStat>);
