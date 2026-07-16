//! Authentication and authorization for TempoForge AI.

pub mod api_key;
pub mod claims;
pub mod clerk;
pub mod rbac;
pub mod secrets;

pub use api_key::{ApiKeyHasher, ApiKeyRecord, generate_api_key};
pub use claims::{AuthContext, Role};
pub use clerk::{ClerkAuth, ClerkConfig};
pub use rbac::{Permission, authorize};
pub use secrets::{SecretBox, encrypt_secret, decrypt_secret};
