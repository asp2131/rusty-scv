pub mod database;
pub mod models;
pub mod github;

pub use database::Database;
pub use models::{Class, Student}; // Removed unused StudentWithClass