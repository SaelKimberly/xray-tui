#![allow(
    clippy::manual_let_else,
    clippy::redundant_pub_crate,
    clippy::wildcard_imports,
    reason = "pub(crate) items in private submodules document internal visibility intent; wildcard imports in private modules are fine"
)]

pub mod error;
pub mod models;
pub mod schema;

pub use error::{DatabaseError, ProfileWithDetails, Result};
pub use database::Database;

mod columns;
mod convert;
mod database;
mod helpers;
mod inner;
