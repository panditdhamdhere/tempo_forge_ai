//! Shared primitives used across TempoForge crates.

pub mod error;
pub mod ids;
pub mod pagination;
pub mod response;
pub mod time;

pub use error::{AppError, AppResult};
pub use ids::*;
pub use pagination::{CursorPage, Page, PageParams};
pub use response::ApiResponse;
pub use time::UtcDateTime;
