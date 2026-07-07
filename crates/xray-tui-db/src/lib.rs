pub mod error;
pub mod models_toasty;
pub use models_toasty as models;
pub use database::Database;
pub use error::{DatabaseError, ProfileWithDetails, Result};

mod database;
