/// Dealing with installation of addons
pub mod addon;
/// Reading and interpreting the user's config files
pub mod config;
/// Operating on instances
pub mod instance;
/// Operating on profiles
pub mod profile;

/// Types and structs for IDs of and references to things
pub mod id {
	use std::fmt::Display;

	use crate::RcType;

	/// The ID for an instance
	pub type InstanceID = RcType<str>;

	/// The ID for a profile
	pub type ProfileID = RcType<str>;

	/// A scoped reference to an instance with a profile ID
	#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
	pub struct InstanceRef {
		/// The ID of the profile
		pub profile: ProfileID,
		/// The ID of the instance
		pub instance: InstanceID,
	}

	impl InstanceRef {
		/// Create a new InstanceRef
		pub fn new(profile: ProfileID, instance: InstanceID) -> Self {
			Self { profile, instance }
		}

		/// Parse an InstanceRef from a string
		pub fn parse(string: String) -> Option<Self> {
			let mut split = string.split(':');
			let profile = split.nth(0)?;
			let instance = split.nth(0)?;
			Some(Self::new(profile.into(), instance.into()))
		}
	}

	impl Display for InstanceRef {
		fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
			write!(f, "{}:{}", self.profile, self.instance)
		}
	}
}
