//! Note: The asynchronous functions in this library expect the use of the Tokio runtime and may panic
//! if it is not used

/// Download utilities
pub mod download;
/// Interacting with the Modrinth API
pub mod modrinth;
/// Interacting with the Smithed API
pub mod smithed;
