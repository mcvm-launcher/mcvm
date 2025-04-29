use mcvm_auth::mc::ClientId;
use mcvm_shared::UpdateDepth;

use crate::util::secrets::get_ms_client_id;

macro_rules! builder_method {
	($name:ident, $ty:ty, $doc:literal) => {
		#[doc = $doc]
		pub fn $name(mut self, $name: $ty) -> Self {
			self.config.$name = $name;
			self
		}
	};
}

/// Configuration for different parts of the core library
pub struct Configuration {
	/// The Microsoft client ID to use for Microsoft authentication
	pub(crate) ms_client_id: ClientId,
	/// The depth of updates to perform
	pub(crate) update_depth: UpdateDepth,
	/// Whether to censor user credentials in output messages and logs
	pub(crate) censor_secrets: bool,
	/// Whether to use file copies instead of hardlinks. Useful if you
	/// are on a filesystem that doesn't like hardlinks
	pub(crate) disable_hardlinks: bool,
	/// Launcher branding
	pub(crate) branding: BrandingProperties,
}

impl Default for Configuration {
	fn default() -> Self {
		Self::new()
	}
}

impl Configuration {
	/// Construct the default configuration
	pub fn new() -> Self {
		Self {
			ms_client_id: get_ms_client_id(),
			update_depth: UpdateDepth::Full,
			censor_secrets: true,
			disable_hardlinks: false,
			branding: BrandingProperties::default(),
		}
	}

	/// Get a builder for the configuration
	pub fn builder() -> ConfigBuilder {
		ConfigBuilder::new()
	}
}

/// Simple builder for the configuration
pub struct ConfigBuilder {
	config: Configuration,
}

impl ConfigBuilder {
	/// Start a new ConfigBuilder with default configuration
	pub fn new() -> Self {
		Self {
			config: Configuration::new(),
		}
	}

	/// Finish building and get the configuration
	pub fn build(self) -> Configuration {
		self.config
	}

	builder_method!(
		ms_client_id,
		ClientId,
		"Set the Microsoft client ID to use for Microsoft / Xbox Live authentication"
	);

	builder_method!(update_depth, UpdateDepth, "Set the depth at which to update files and versions");

	builder_method!(
		censor_secrets,
		bool,
		"Set whether to censor user credentials in output messages and logs"
	);

	builder_method!(
		disable_hardlinks,
		bool,
		"Set whether to disable the use of hardlinks"
	);

	builder_method!(branding, BrandingProperties, "Set the branding properties");
}

impl Default for ConfigBuilder {
	fn default() -> Self {
		Self::new()
	}
}

/// Branding properties for the launcher to send to the client
pub struct BrandingProperties {
	/// The desired launcher name to send to the client
	pub(crate) launcher_name: String,
	/// The desired launcher version to send to the client
	pub(crate) launcher_version: String,
}

impl Default for BrandingProperties {
	fn default() -> Self {
		Self {
			launcher_name: "mcvm_core".into(),
			launcher_version: env!("CARGO_PKG_VERSION").into(),
		}
	}
}

impl BrandingProperties {
	/// Create new BrandingProperties
	pub fn new(name: String, version: String) -> Self {
		Self {
			launcher_name: name,
			launcher_version: version,
		}
	}
}
