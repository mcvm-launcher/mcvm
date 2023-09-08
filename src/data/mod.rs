/// Dealing with installation of addons
pub mod addon;
/// Reading and interpreting the user's config files
pub mod config;
/// Operating on instances
pub mod instance;
/// Operating on profiles
pub mod profile;
/// Operating on users
pub mod user;

/// Types for IDs of things
pub mod id {
	use std::rc::Rc;

	/// The ID for an instance
	pub type InstanceID = Rc<str>;

	/// The ID for a profile
	pub type ProfileID = Rc<str>;
}
