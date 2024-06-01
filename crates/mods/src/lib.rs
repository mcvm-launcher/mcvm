#![warn(missing_docs)]

//! This library can install different game modifications, such as Fabric and Paper,
//! for use with the mcvm_core library. It is packaged separately so that users who
//! do not want this functionality and only act on the vanilla game don't have to include
//! them
//!
//! Note: The functions in this library expect the use of the Tokio runtime and may panic
//! if it is not used

/// Installation of the Fabric and Quilt modloaders
pub mod fabric_quilt;
/// Installation of projects from PaperMC, such as the Paper and Folia servers
pub mod paper;
/// Installation of SpongeVanilla
pub mod sponge;
