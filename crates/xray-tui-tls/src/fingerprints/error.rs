//! Fingerprint selection/build errors.

/// Strict-resolution and builder failures.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum FingerprintError {
    /// No resolution-table row satisfies the query.
    #[error("unknown fingerprint combination `{query}`; available: {}", available.join(", "))]
    Unknown {
        /// Rendered query (`chrome@133/windows/desktop` shape).
        query: String,
        /// Resolvable identity names, sorted.
        available: Vec<String>,
    },
    /// A builder override violates a base-spec invariant.
    #[error("invalid fingerprint override: {0}")]
    InvalidOverride(String),
}
