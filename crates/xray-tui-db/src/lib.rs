pub mod error;
pub mod models_toasty;
pub use database::Database;
pub use error::{DatabaseError, ProfileWithDetails, Result};
pub use models_toasty as models;

mod database;
