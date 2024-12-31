#![warn(missing_docs)]

//! This is the library for MCVM and pretty much all of the features that the
//! CLI uses.
//!
//! Note: The asynchronous functions in this library expect the use of the Tokio runtime and may panic
//! if it is not used
//!
//! # Features
//!
//! - `arc`: MCVM uses Rc's in a couple places. Although these are more performant than Arc's, they
//! may not be compatible with some async runtimes. With this feature enabled, these Rc's will be replaced with
//! Arc's where possible.
//! - `builder`: Enable or disable the config builder system, which isn't needed if you are just deserializing the standard config.
//! - `disable_profile_update_packages`: A workaround for `https://github.com/rust-lang/rust/issues/102211`. If you are
//! getting higher-ranked lifetime errors when running the update_profiles function, try enabling this. When enabled, the
//! update_profiles function will no longer update packages at all.
//! - `schema`: Enable generation of JSON schemas using the `schemars` crate

pub use mcvm_core as core;
pub use mcvm_parse as parse;
pub use mcvm_pkg as pkg_crate;
pub use mcvm_plugin as plugin_crate;
pub use mcvm_shared as shared;

/// Installable addons
pub mod addon;
/// MCVM configuration
pub mod config;
/// Launchable instances
pub mod instance;
/// File and data format input / output
pub mod io;
/// Dealing with packages
pub mod pkg;
/// Plugin-related things, like loading, configuration, and management/installation
pub mod plugin;
/// Configuration profiles for instances
pub mod profile;
/// Common utilities that can't live anywhere else
pub mod util;

/// The global struct used as an Rc, depending on the `arc` feature
#[cfg(feature = "arc")]
pub type RcType<T> = std::sync::Arc<T>;
/// The global struct used as an Rc, depending on the `arc` feature
#[cfg(not(feature = "arc"))]
pub type RcType<T> = std::rc::Rc<T>;

/// The version of MCVM
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
