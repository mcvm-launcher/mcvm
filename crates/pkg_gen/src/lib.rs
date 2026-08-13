#![warn(missing_docs)]

//! Package generation for Nitrolaunch from platforms like Modrinth and Smithed. Used by the package generation plugin and repository provider plugins.

/// CurseForge package generation
pub mod curse;
/// Modrinth package generation
pub mod modrinth;
/// Substitution for relations in generated packages
pub mod relation_substitution;
/// Smithed package generation
pub mod smithed;
